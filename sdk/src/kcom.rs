use num_enum::TryFromPrimitive;
use rp_pac::SIO;

/// Type of Kernel Communication Message
#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum KComType {
    Syscall = 0,
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
