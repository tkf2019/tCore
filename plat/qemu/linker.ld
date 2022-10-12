OUTPUT_ARCH(riscv)

ENTRY(_start)

SECTIONS
{
    . = 0x80200000;
    
    .text : {
        *(.text.entry)
        . = ALIGN(0x1000);
        *(.text.trampoline);
        . = ALIGN(0x1000);
        *(.text .text.*)
        
        PROVIDE(etext = .);
    }

    . = ALIGN(0x1000);
    
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(0x1000);
    
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(0x1000);
    
    .bss : {
        *(.bss.stack)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);

    PROVIDE(end = .);

    /DISCARD/ : {
        *(.eh_frame)
    }
}