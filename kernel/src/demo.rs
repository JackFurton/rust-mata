//! A deliberate fault, so the reporting in `fault` can be checked rather than
//! eyeballed. Built only under the `fault-demo` feature.

use crate::debug;

/// `udf` is the architecturally undefined instruction. It raises UsageFault
/// with CFSR.UNDEFINSTR, and the stacked PC points at the instruction itself
/// rather than past it, which is what makes it usable as a fixture.
///
/// Naked so that `udf` is the first instruction at this symbol. A plain
/// function gets a frame-pointer prologue at `opt-level = 1`, which put the
/// fault two instructions in and made the expected address profile-dependent.
#[unsafe(naked)]
pub extern "C" fn fault_site() -> ! {
    core::arch::naked_asm!("udf #0")
}

/// Called from the fault report with the stacked PC.
pub fn check(pc: u32) -> ! {
    // A function's address in Rust carries the thumb bit, since that is what a
    // branch needs to stay in thumb state. The core strips it before stacking.
    let site: extern "C" fn() -> ! = fault_site;
    let expected = (site as usize as u32) & !1;

    if pc == expected {
        crate::kprintln!("demo: pc matches fault_site");
        debug::exit(0)
    }

    crate::kprintln!("demo: pc {pc:#010x} is not fault_site {expected:#010x}");
    debug::exit(1)
}
