//! Debug output over ARM semihosting.
//!
//! `bkpt #0xAB` traps to whatever is supervising the core (QEMU here, a
//! debugger on real silicon) which then performs the I/O on our behalf. It
//! needs no peripheral, so it works before any clock or UART is configured,
//! which is why the earliest boot code can talk. It is also synchronous and
//! very slow, and it hangs the core if nothing is supervising, so it stays a
//! bring-up tool rather than a logging transport.

use core::fmt::{self, Write};

const SYS_WRITEC: u32 = 0x03;
const SYS_EXIT_EXTENDED: u32 = 0x20;
const APP_EXIT: u32 = 0x2_0026;

unsafe fn call(op: u32, arg: u32) -> u32 {
    let mut ret = op;
    unsafe {
        core::arch::asm!(
            "bkpt #0xAB",
            inout("r0") ret,
            in("r1") arg,
            options(nostack, preserves_flags),
        );
    }
    ret
}

struct Semihost;

impl Write for Semihost {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            unsafe { call(SYS_WRITEC, &raw const byte as u32) };
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments<'_>) {
    let _ = Semihost.write_fmt(args);
}

pub fn println(s: &str) {
    let _ = Semihost.write_str(s);
    let _ = Semihost.write_str("\n");
}

/// Stop the machine, reporting `code` to the supervisor. Under QEMU this exits
/// with that status, which is how the integration tests pass or fail.
pub fn exit(code: u32) -> ! {
    let block = [APP_EXIT, code];
    unsafe { call(SYS_EXIT_EXTENDED, &raw const block as u32) };

    // Reached only when nothing is listening for the exit request.
    loop {
        unsafe { core::arch::asm!("wfi", options(nomem, nostack)) };
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => { $crate::debug::print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::debug::println("") };
    ($($arg:tt)*) => { $crate::debug::print(format_args!("{}\n", format_args!($($arg)*))) };
}
