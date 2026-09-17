//! Exception reporting.
//!
//! When the core takes an exception it pushes eight words of caller state onto
//! the stack that was active at the time, then enters the handler. Those words
//! are the only record of where the fault came from: by the time a normal Rust
//! function runs, its own prologue has already moved the stack pointer. So the
//! entry point has to be naked, capture the pointer untouched, and hand it on.

/// System Control Block registers, all documented in ARM DDI 0403 B3.2.
mod scb {
    pub const CFSR: *const u32 = 0xE000_ED28 as *const u32;
    pub const HFSR: *const u32 = 0xE000_ED2C as *const u32;
    pub const MMFAR: *const u32 = 0xE000_ED34 as *const u32;
    pub const BFAR: *const u32 = 0xE000_ED38 as *const u32;
}

/// The eight words the core stacks on exception entry, in push order. Floating
/// point state would follow on a core with an FPU; the M3 has none.
#[repr(C)]
struct ExceptionFrame {
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    lr: u32,
    pc: u32,
    xpsr: u32,
}

/// Shared entry point for every exception vector.
///
/// Naked because the frame pointer must be read before anything can disturb
/// the stack. Bit 2 of EXC_RETURN, which the core left in LR, says which stack
/// the frame is on: clear means the interrupted code was using MSP, set means
/// PSP. Both matter as soon as tasks get their own stacks.
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fault_entry() {
    core::arch::naked_asm!(
        "mrs r0, msp",
        "tst lr, #4",
        "beq 2f",
        "mrs r0, psp",
        "2:",
        "mrs r1, ipsr", // which exception; 0 would mean thread mode
        "b {report}",
        report = sym report,
    )
}

/// Decode a bitfield into the flag names that are set.
fn print_flags(value: u32, flags: &[(u32, &str)]) {
    for (bit, name) in flags {
        if value & (1 << bit) != 0 {
            crate::kprint!(" {name}");
        }
    }
}

const MEM_MANAGE_FLAGS: &[(u32, &str)] = &[
    (0, "iaccviol"),
    (1, "daccviol"),
    (3, "munstkerr"),
    (4, "mstkerr"),
    (5, "mlsperr"),
];
const BUS_FAULT_FLAGS: &[(u32, &str)] = &[
    (8, "ibuserr"),
    (9, "preciserr"),
    (10, "impreciserr"),
    (11, "unstkerr"),
    (12, "stkerr"),
    (13, "lsperr"),
];
const USAGE_FAULT_FLAGS: &[(u32, &str)] = &[
    (16, "undefinstr"),
    (17, "invstate"),
    (18, "invpc"),
    (19, "nocp"),
    (24, "unaligned"),
    (25, "divbyzero"),
];
const HARD_FAULT_FLAGS: &[(u32, &str)] = &[(1, "vecttbl"), (30, "forced"), (31, "debugevt")];

const MMFAR_VALID: u32 = 1 << 7;
const BFAR_VALID: u32 = 1 << 15;

fn exception_name(number: u32) -> &'static str {
    match number {
        2 => "NMI",
        3 => "HardFault",
        4 => "MemManage",
        5 => "BusFault",
        6 => "UsageFault",
        11 => "SVCall",
        14 => "PendSV",
        15 => "SysTick",
        _ => "unknown",
    }
}

/// A fault raised while this is running cannot be reported: HardFault already
/// runs at priority -1, so the core locks up rather than re-entering the
/// handler, and the only output is QEMU's own `Lockup: can't escalate 3 to
/// HardFault`. Everything below therefore reads memory it was handed and does
/// nothing that could fault on its own.
extern "C" fn report(frame: *const ExceptionFrame, exception: u32) -> ! {
    // Safe as long as the stacking itself succeeded. It may not have: a fault
    // while pushing the frame sets CFSR.STKERR and leaves this pointing at
    // whatever the stack pointer had reached, so the values below are evidence
    // rather than truth.
    let frame = unsafe { &*frame };

    let cfsr = unsafe { scb::CFSR.read_volatile() };
    let hfsr = unsafe { scb::HFSR.read_volatile() };

    crate::kprintln!(
        "\nfault: {} (exception {})",
        exception_name(exception),
        exception
    );
    crate::kprintln!("  pc    {:#010x}", frame.pc);
    crate::kprintln!("  lr    {:#010x}", frame.lr);
    crate::kprintln!("  xpsr  {:#010x}", frame.xpsr);
    crate::kprintln!(
        "  r0-r3 {:#010x} {:#010x} {:#010x} {:#010x}",
        frame.r0,
        frame.r1,
        frame.r2,
        frame.r3
    );
    crate::kprintln!("  r12   {:#010x}", frame.r12);

    // FORCED means the fault escalated: a MemManage, BusFault or UsageFault
    // arrived with its own handler disabled, so it came here instead. CFSR
    // still names the original cause.
    crate::kprint!("  hfsr  {hfsr:#010x}");
    print_flags(hfsr, HARD_FAULT_FLAGS);
    crate::kprintln!();

    crate::kprint!("  cfsr  {cfsr:#010x}");
    print_flags(cfsr, MEM_MANAGE_FLAGS);
    print_flags(cfsr, BUS_FAULT_FLAGS);
    print_flags(cfsr, USAGE_FAULT_FLAGS);
    crate::kprintln!();

    // The address registers only hold a fault address when their valid bit is
    // set; otherwise they are stale from an earlier fault. Neither branch is
    // reachable on QEMU's lm3s6965evb, which returns data for unassigned
    // addresses instead of raising a BusFault; the MPU will make them live.
    if cfsr & MMFAR_VALID != 0 {
        crate::kprintln!("  mmfar {:#010x}", unsafe { scb::MMFAR.read_volatile() });
    }
    if cfsr & BFAR_VALID != 0 {
        crate::kprintln!("  bfar  {:#010x}", unsafe { scb::BFAR.read_volatile() });
    }

    #[cfg(feature = "fault-demo")]
    crate::demo::check(frame.pc);

    #[cfg(not(feature = "fault-demo"))]
    halt()
}

/// Spin clock-gated rather than resetting, so a debugger attaching after the
/// fault still finds the stack and registers intact.
pub fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("wfi", options(nomem, nostack)) };
    }
}
