OUTPUT_ARCH(riscv)
ENTRY(_start)

/* PROVIDE(stext=0x80000000); */

BASE_ADDR = 0x80000000;

SECTIONS
{
  . = BASE_ADDR;
  PROVIDE(start = .);

  .text : {
    stext = .;
    *(.text.entry)
    *(.text .text.*)
    . = ALIGN(8);
    etext = .;
  }

  .rodata : ALIGN(8) {
    srodata = .;
    *(.rodata .rodata.*)
    *(.srodata .srodata.*)
    . = ALIGN(8);
    erodata = .;
  }

  .data : ALIGN(8) {
    PROVIDE(ldata = LOADADDR(.data));
    sdata = .;
    *(.data .data.*)
    *(.sdata .sdata.*)
    . = ALIGN(8);
    PROVIDE(edata = .);
  }

  .bss (NOLOAD) : ALIGN(8) {
    PROVIDE(sbss = .);
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    . = ALIGN(8);
    PROVIDE(ebss = .);
  }

  PROVIDE(stack = ALIGN(ebss + 262144 - 7, 8));
  PROVIDE(end = .);
}