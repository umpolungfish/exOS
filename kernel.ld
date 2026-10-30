/* kernel.ld — exoterikOS linker script
 * Single RWX LOAD segment at -2GB (top-of-48-bit), static executable.
 * bootloader 0.9 x86_64 maps the ELF LOAD segments; a single segment
 * avoids the multi-segment PageAlreadyMapped issue with the bootloader's
 * RecursivePageTable.
 */

ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(7);   /* RWX — bootloader does not use NX flag */
}

SECTIONS {
    . = 0xFFFFFFFF80100000;

    .text : ALIGN(4096) {
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(COMMON)
        *(.bss .bss.*)
    } :text

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.comment)
    }
}
