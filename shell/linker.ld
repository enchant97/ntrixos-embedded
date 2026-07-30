ENTRY(_start)

MEMORY {
    FLASH : ORIGIN = 0x10040000, LENGTH = 1792K
    RAM : ORIGIN = 0x20020000, LENGTH = 128K
    /* Shared memory for storing Syscall message */
    /* NOTE: not all 4K is actually used */
    SYSCALL_MSG : ORIGIN = 0x20040000, LENGTH = 4K
}

SECTIONS
{
    . = 0x10040000;

    .text : {
        KEEP(*(.text._start))
        *(.text*)
    } > FLASH

    .rodata : { *(.rodata*) } > FLASH

    .data : {
        _data_start = .;
        *(.data*)
        _data_end = .;
    } > RAM AT > FLASH

    _data_load = LOADADDR(.data);

    .bss : {
        _bss_start = .;
        *(.bss*)
        *(COMMON)
        _bss_end = .;
    } > RAM

    /DISCARD/ : {
        *(.comment*)
        *(.note*)
        *(.ARM.exidx*)
        *(.ARM.extab*)
        *(.ARM.attributes*)
    }

    .syscall_message : {
        KEEP(*(.syscall_message))
    } > SYSCALL_MSG
}
