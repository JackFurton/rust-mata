//! Reset path: vector table, RAM initialisation, entry into the kernel.

use crate::fault::fault_entry;

unsafe extern "C" {
    static mut __sbss: u32;
    static mut __ebss: u32;
    static mut __sdata: u32;
    static mut __edata: u32;
    static __sidata: u32;
}

/// One entry in the vector table. Slots the architecture reserves hold a zero
/// word rather than an address, so the two cases need the same-sized union.
#[repr(C)]
union Vector {
    handler: unsafe extern "C" fn(),
    reserved: u32,
}

#[used]
#[unsafe(link_section = ".vector_table.reset")]
static RESET_VECTOR: unsafe extern "C" fn() -> ! = reset;

/// Exceptions 2..=15. Entry 0 is the initial SP and entry 1 is the reset
/// vector, both emitted ahead of this array by link.x.
#[used]
#[unsafe(link_section = ".vector_table.exceptions")]
#[rustfmt::skip]
static EXCEPTIONS: [Vector; 14] = [
    Vector { handler: fault_entry }, //  2 NMI
    Vector { handler: fault_entry }, //  3 HardFault
    Vector { handler: fault_entry }, //  4 MemManage
    Vector { handler: fault_entry }, //  5 BusFault
    Vector { handler: fault_entry }, //  6 UsageFault
    Vector { reserved: 0 },          //  7
    Vector { reserved: 0 },          //  8
    Vector { reserved: 0 },          //  9
    Vector { reserved: 0 },          // 10
    Vector { handler: fault_entry }, // 11 SVCall
    Vector { reserved: 0 },          // 12
    Vector { reserved: 0 },          // 13
    Vector { handler: fault_entry }, // 14 PendSV
    Vector { handler: fault_entry }, // 15 SysTick
];

/// First instruction executed after reset, with SP already loaded from the
/// vector table. Nothing initialised yet: no statics, no heap, no stack frame
/// worth trusting from before the reset.
#[unsafe(no_mangle)]
unsafe extern "C" fn reset() -> ! {
    unsafe {
        // Zero .bss and copy .data out of flash before any Rust code can
        // observe a static. Both regions are word-aligned and word-sized by
        // link.x, so these counts are in words, not bytes.
        let sbss = &raw mut __sbss;
        let bss_words = (&raw mut __ebss).offset_from(sbss) as usize;
        core::ptr::write_bytes(sbss, 0, bss_words);

        let sdata = &raw mut __sdata;
        let data_words = (&raw mut __edata).offset_from(sdata) as usize;
        core::ptr::copy_nonoverlapping(&raw const __sidata, sdata, data_words);
    }

    crate::kernel_main()
}
