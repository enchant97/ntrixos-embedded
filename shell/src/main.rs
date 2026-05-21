#![no_std]
#![no_main]

use libsys::{ExitCode, nostd::io::Write};

// Symbols injected by the linker script
unsafe extern "C" {
    static mut _data_start: u32;
    static mut _data_end: u32;
    static _data_load: u32; // LMA — read only, lives in flash
    static mut _bss_start: u32;
    static mut _bss_end: u32;
}

unsafe fn init_memory() {
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

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(abi: *const libsys::KernelAbi) -> ExitCode {
    unsafe {
        init_memory();
    }
    libsys::core::sys_init(abi);
    libsys::display::display().lock(|d| {
        d.set_display_mode(libsys::display::DisplayMode::Text)
            .expect("failed to set mode");
        d.get_framebuffer_mut(|mut fb| {
            let msg = b"Hello World!";
            fb.write_all(msg).unwrap();
        });
        d.flush().expect("fail to flush display buffer");
    });
    ExitCode::Ok
}
