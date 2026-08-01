use embassy_rp::pac::SIO;
use num_enum::TryFromPrimitive;
use sdk::kcom::KComType;

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

pub fn write_kcom_fifo_blocking(kcom_type: KComType) {
    let word = kcom_type as u32;
    SIO.fifo().wr().write_value(word);
    cortex_m::asm::sev();
}

pub fn read_kcom_fifo_blocking() -> Option<KComType> {
    let word = SIO.fifo().rd().read();
    KComType::try_from_primitive(word).ok()
}
