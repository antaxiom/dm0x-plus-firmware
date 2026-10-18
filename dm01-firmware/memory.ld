/* Linker script for ToSeven DM01 Cortex-M4F Firmware (loads at 0x08001800 above factory bootloader) */
MEMORY
{
  FLASH (rx) : ORIGIN = 0x08001800, LENGTH = 58K
  RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 32K
}

ENTRY(Reset_Handler)

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    KEEP(*(.vector_table))
  } > FLASH

  .text : ALIGN(4)
  {
    *(.text .text.*)
    *(.rodata .rodata.*)
  } > FLASH

  .data : ALIGN(4)
  {
    *(.data .data.*)
  } > RAM AT > FLASH

  .bss : ALIGN(4)
  {
    *(.bss .bss.*)
  } > RAM

  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*)
  }
}
