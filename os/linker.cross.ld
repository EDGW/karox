OUTPUT_ARCH(%ARCH%)
ENTRY(_start)

SECTIONS
{
    .text : ALIGN(4K) {
        _skernel = .;
        _stext = .;
        *(.text.entry)

        . = ALIGN(4K);
        *(.text .text.*)
        . = ALIGN(4K);
        _etext = .;
    } > VIRT AT> RAM

    .rodata : ALIGN(4K) {
        _srodata = .;
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        . = ALIGN(4K);
        _erodata = .;
    } > VIRT AT> RAM

    .data : ALIGN(4K) {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        *(.got)
        . = ALIGN(4K);
        _edata = .;
    } > VIRT AT> RAM

    .bss : ALIGN(4K) {
        _sbss = .;
        *(.bss.stack)
        *(.bss.heap)
        _kbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        . = ALIGN(4K);
        _ebss = .;
    } > VIRT AT> RAM

    . = ALIGN(4K);
    _ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}

MEMORY
{
    VIRT : ORIGIN = %VIRT_START%, LENGTH = 300M
    RAM : ORIGIN = %RAM_START%, LENGTH = 300M
}

