ENTRY(entry);

MEMORY
{
  FLAG : ORIGIN = 0x10000, LENGTH = 1024K
}


SECTIONS {

    .text : ALIGN(4096) {
        KEEP(*(.entry))
        *(.text .text.*)
    } > FLAG

    .rodata :
    {
        *(.rodata .rodata.*);
    } > FLAG

    .bss :
    {
        *(.bss .bss.*);
    } > FLAG

    .data :
    {
        *(.data .data.*);
    } > FLAG
}
