#![no_std]
#![no_main]

mod boot;
mod debug;
mod fault;
mod panic;

#[cfg(feature = "fault-demo")]
mod demo;

/// Witnesses for the two halves of RAM setup. Read volatile so the compiler
/// cannot fold them back into the constants it can see here: the point is to
/// observe what is actually in RAM after reset ran.
static mut INITIALISED: u32 = 0xC0FF_EE00;
static mut ZEROED: u32 = 0;

fn kernel_main() -> ! {
    let initialised = unsafe { core::ptr::read_volatile(&raw const INITIALISED) };
    let zeroed = unsafe { core::ptr::read_volatile(&raw const ZEROED) };

    kprintln!("mata: boot");
    kprintln!("  .data  {initialised:#010x} (expect 0xc0ffee00)");
    kprintln!("  .bss   {zeroed:#010x} (expect 0x00000000)");

    #[cfg(feature = "fault-demo")]
    demo::fault_site();

    #[cfg(not(feature = "fault-demo"))]
    debug::exit(0)
}
