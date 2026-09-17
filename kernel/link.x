/* lm3s6965evb (Stellaris LM3S6965, Cortex-M3): 256K flash at 0, 64K SRAM. */
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 256K
  RAM   (rwx) : ORIGIN = 0x20000000, LENGTH = 64K
}

/* The core loads SP from the first vector before executing anything, so the
   stack has to be placed here rather than by any code we write. It grows down
   from the top of RAM toward .bss. */
__stack_top = ORIGIN(RAM) + LENGTH(RAM);

ENTRY(reset);

SECTIONS
{
  /* Must land at address 0: the core reads SP and PC from here on reset, and
     later reads handler addresses from it on every exception. */
  .vector_table ORIGIN(FLASH) :
  {
    LONG(__stack_top);
    KEEP(*(.vector_table.reset));
    KEEP(*(.vector_table.exceptions));
  } > FLASH

  .text :
  {
    *(.text .text.*);
    . = ALIGN(4);
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
    . = ALIGN(4);
  } > FLASH

  /* Lives in RAM at runtime but ships in flash; reset copies it across. */
  .data : ALIGN(4)
  {
    __sdata = .;
    *(.data .data.*);
    . = ALIGN(4);
    __edata = .;
  } > RAM AT > FLASH

  __sidata = LOADADDR(.data);

  .bss (NOLOAD) : ALIGN(4)
  {
    __sbss = .;
    *(.bss .bss.*);
    . = ALIGN(4);
    __ebss = .;
  } > RAM

  /* Unwind tables for a target with no unwinder. */
  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
    *(.ARM.extab .ARM.extab.*);
  }
}

ASSERT(SIZEOF(.vector_table) == 64, "vector table is not 16 words");
ASSERT(__ebss <= __stack_top, "RAM overflow: .bss collides with the stack");
