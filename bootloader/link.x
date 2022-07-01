ENTRY(entry);

MEMORY
{
  FLAT : ORIGIN = 0x10000, LENGTH = 1024K
}


SECTIONS {

    .text : ALIGN(4096) {
        KEEP(*(.entry))
        *(.text .text.*)
    } > FLAT

    .rodata :
    {
        *(.rodata .rodata.*);
    } > FLAT

    .bss :
    {
        *(.bss .bss.*);
    } > FLAT

    .data :
    {
        *(.data .data.*);
    } > FLAT
}
