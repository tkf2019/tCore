OUTPUT_ARCH(riscv)

ENTRY(__entry)

SECTIONS
{
    . = 0x80200000;
    skernel = .;
    stext = .;

    .text : {
        *(.text.entry)
        . = ALIGN(0x1000);
        *(.text.trampoline);
        . = ALIGN(0x1000);
        *(.text .text.*)
        
        PROVIDE(etext = .);
    }

    . = ALIGN(0x1000);
    etext = .;
    srodata = .;
    
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(0x1000);
    erodata = .;
    sdata = .;
    
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(0x1000);
    edata = .;
    sbss_with_stack = .;
    
    .bss : {
        *(.bss.stack)
        
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}