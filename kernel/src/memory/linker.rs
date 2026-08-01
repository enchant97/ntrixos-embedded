//! Values provided by the linker.
use sdk::kcom::Syscall;

unsafe extern "C" {
    pub static __shell_flash_start: u32;
    pub static __shell_flash_end: u32;
}

// HACK there must be a better way to include the shell.
#[used]
#[unsafe(link_section = ".shell_flash_slot")]
static SHELL_FLASH: [u8; include_bytes!("../../../target/thumbv6m-none-eabi/bin/shell.bin").len()] =
    *include_bytes!("../../../target/thumbv6m-none-eabi/bin/shell.bin");

#[unsafe(link_section = ".syscall_message")]
pub static SYSCALL_MESSAGE: Syscall = Syscall::empty();
