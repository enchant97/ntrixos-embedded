MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    /* define kernel memory areas */
    FLASH : ORIGIN = 0x10000100, LENGTH = 256K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
    /* define app section memory areas */
    SHELL_FLASH : ORIGIN = 0x10040000, LENGTH = 1792K
    APP_RAM : ORIGIN = 0x20020000, LENGTH = 128K
    /* Shared memory for storing Syscall message */
    /* NOTE: not all 4K is actually used */
    SYSCALL_MSG : ORIGIN = 0x20040000, LENGTH = 4K
}

SECTIONS {
    __shell_flash_start = ORIGIN(SHELL_FLASH);
    __shell_flash_end   = ORIGIN(SHELL_FLASH) + LENGTH(SHELL_FLASH);

    .shell_flash_slot : {
        KEEP(*(.shell_flash_slot))
    } > SHELL_FLASH

    .syscall_message : {
        KEEP(*(.syscall_message))
    } > SYSCALL_MSG
}
