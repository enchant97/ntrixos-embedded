use num_enum::TryFromPrimitive;
use rp_pac::SIO;
use sdk::kcom::{KComType, Syscall, SyscallNum};

// Symbols injected by the linker script
unsafe extern "C" {
    static mut _data_start: u32;
    static mut _data_end: u32;
    static _data_load: u32; // LMA — read only, lives in flash
    static mut _bss_start: u32;
    static mut _bss_end: u32;
}

#[unsafe(link_section = ".syscall_message")]
pub static SYSCALL_MESSAGE: Syscall = Syscall {
    num: SyscallNum::Null,
    args: [0; 6],
    result: 0,
};

pub fn write_kcom_fifo_blocking(kcom_type: KComType) {
    let word = kcom_type as u32;
    SIO.fifo().wr().write_value(word);
    cortex_m::asm::sev();
}

pub fn read_kcom_fifo_blocking() -> Option<KComType> {
    let word = SIO.fifo().rd().read();
    if let Ok(kcom_type) = KComType::try_from_primitive(word) {
        Some(kcom_type)
    } else {
        None
    }
}

/// Init the app memory, called once on program start.
///
/// # Safety
/// - Must be called once in main function and be the first operation.
/// - Must not be called if using `#[libsys::entrypoint]`
pub unsafe fn init_memory() {
    // copy .data from flash to RAM
    let mut src = &raw const _data_load;
    let mut dst = &raw mut _data_start;
    let end = &raw const _data_end;
    while dst < end as *mut u32 {
        unsafe {
            dst.write_volatile(src.read());
            src = src.add(1);
            dst = dst.add(1);
        }
    }

    // zero .bss
    let mut bss = &raw mut _bss_start;
    let bss_end = &raw const _bss_end;
    while bss < bss_end as *mut u32 {
        unsafe {
            bss.write_volatile(0);
            bss = bss.add(1);
        }
    }
}
