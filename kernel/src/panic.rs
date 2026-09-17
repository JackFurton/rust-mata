use crate::debug;

/// Panics are fatal: nothing here can unwind (`panic = "abort"`, and the
/// unwind tables are discarded by link.x), and there is no supervisor process
/// to restart us.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    crate::kprintln!("panic: {}", info.message());
    if let Some(loc) = info.location() {
        crate::kprintln!("  at {}:{}", loc.file(), loc.line());
    }
    debug::exit(1)
}
