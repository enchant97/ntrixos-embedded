use crate::common::AppEntry;

pub mod linker;

/// Gets the shell app entry function.
pub fn get_shell_app_entry() -> AppEntry {
    unsafe {
        // `| 1` enables Thumb mode
        let addr = &raw const linker::__shell_flash_start as usize | 1;
        core::mem::transmute(addr)
    }
}
