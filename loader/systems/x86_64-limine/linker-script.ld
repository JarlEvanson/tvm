OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86_64)

ENTRY(loader_main)

PHDRS {
    rodata      PT_LOAD             FLAGS(4)    ;
    text        PT_LOAD             FLAGS(1 | 4);
    data        PT_LOAD             FLAGS(2 | 4);
    dynamic     PT_DYNAMIC                      ;
}

SECTIONS {
    . += ALIGN(CONSTANT(COMMONPAGESIZE)) - SIZEOF_HEADERS;

    phdrs_start = .;
    . += SIZEOF_HEADERS;
    phdrs_end = .;

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .text ALIGN(CONSTANT(COMMONPAGESIZE)): {
        *(.text .text.*)
    } :text

    .data ALIGN(CONSTANT(COMMONPAGESIZE)) : {
        *(.data .data.*)

        KEEP(*(.limine_requests_start))
        KEEP(*(.limine_base_revision_tag))
        KEEP(*(.limine_requests))
        KEEP(*(.limine_requests_end))
    } :data

    .got ALIGN(CONSTANT(COMMONPAGESIZE)) : {
        *(.got .got.*)
    } :data

    .dynamic : {
        *(.dynamic .dynamic.*)
    } :dynamic :data

    .bss : {
        *(.bss .bss.*)
    } :data
}
