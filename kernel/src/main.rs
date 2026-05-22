#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use assign_resources::assign_resources;
use core::ptr::{addr_of_mut, null_mut};
use core::{ffi::c_void, str};
use embassy_executor::Executor;
use embassy_futures::join::join;
use embassy_rp::peripherals::USB;
use embassy_rp::{
    Peri, bind_interrupts, gpio,
    multicore::Stack,
    peripherals::{self, DMA_CH0},
    spi::{self, Spi},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use embassy_time::Delay;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::{
    Drawable,
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    prelude::{Point, Size},
    text::{Alignment, Text, renderer::TextRenderer},
};
use ibm437::IBM437_8X8_REGULAR;
use sdk::{ExitCode, FileDescriptor, KernelAbi};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::common::AppEntry;
use crate::drivers::display::ST7920;
use crate::memory::get_shell_app_entry;

mod common;
mod drivers;
mod memory;

static mut APP_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

pub static KERNEL_ABI: KernelAbi = KernelAbi {
    get_version: abi_get_version,
    exit: abi_exit,
    malloc: abi_malloc,
    free: abi_free,
    write: abi_write,
    read: abi_read,
    flush: abi_flush,
    seek: abi_seek,
    mmap: abi_mmap,
    ioctl: abi_ioctl,
};

/// Used to signal that the current app has finished.
pub static APP_EXIT_SIG: Signal<CriticalSectionRawMutex, ExitCode> = Signal::new();
/// Used to signal which app to launch.
pub static APP_LAUNCH_SIG: Signal<CriticalSectionRawMutex, AppEntry> = Signal::new();

static FLUSH_DISPLAY_SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();
// NOTE "static mut" is generally not a good idea. But may be required for this kernel?
// display buffer that either stores text or pixel data, depending on current mode.
static mut DISPLAY_FB: [u8; 128 * 64] = [0; 128 * 64];

const MAX_KEYBOARD_EVENTS_BACKLOG: usize = 6;
static KEYBOARD_EVENT_CHANNEL: Channel<
    CriticalSectionRawMutex,
    sdk::drivers::keyboard::KeyEvent,
    MAX_KEYBOARD_EVENTS_BACKLOG,
> = Channel::new();

assign_resources! {
    display: DisplayResources {
        spi: SPI0,
        cs: PIN_17,
        sck: PIN_18,
        mosi: PIN_19,
        dma: DMA_CH0,
    },
    usb: UsbResources {
        usb: USB,
    },
}

bind_interrupts!(struct Irqs{
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>;
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

extern "C" fn abi_get_version() -> u32 {
    1
}

extern "C" fn abi_exit(code: ExitCode) -> ! {
    todo!()
}

extern "C" fn abi_malloc(size: usize) -> *mut u8 {
    todo!()
}

extern "C" fn abi_free(ptr: *mut u8) {
    todo!()
}

extern "C" fn abi_write(fd: FileDescriptor, buff: *const u8, buff_len: usize) {
    todo!()
}

extern "C" fn abi_read(fd: FileDescriptor, buff: *mut u8, buff_len: usize) -> isize {
    match fd {
        FileDescriptor::Display => -1,
        FileDescriptor::KeyEvents => {
            use sdk::drivers::keyboard::KeyEvent;
            if buff_len < size_of::<KeyEvent>() {
                // buffer too small
                return -1;
            }
            // TODO this ignores a buffer with more than 1 event space
            //      only ever returns 1 event
            let event: KeyEvent;
            loop {
                if let Ok(v) = KEYBOARD_EVENT_CHANNEL.try_receive() {
                    event = v;
                    break;
                }
                cortex_m::asm::wfe();
            }
            unsafe {
                (buff as *mut KeyEvent).write(event);
            }
            size_of::<KeyEvent>() as isize
        }
    }
}

extern "C" fn abi_flush(fd: FileDescriptor) {
    match fd {
        FileDescriptor::Display => FLUSH_DISPLAY_SIG.signal(()),
        FileDescriptor::KeyEvents => {}
    }
}

extern "C" fn abi_seek(fd: FileDescriptor, offset: usize) {
    todo!()
}

extern "C" fn abi_mmap(fd: FileDescriptor) -> *mut c_void {
    match fd {
        FileDescriptor::Display => unsafe { DISPLAY_FB.as_mut_ptr() as *mut c_void },
        FileDescriptor::KeyEvents => null_mut() as *mut c_void,
    }
}

extern "C" fn abi_ioctl(
    fd: FileDescriptor,
    op: usize,
    in_arg: *const c_void,
    out_arg: *mut c_void,
) -> ExitCode {
    ExitCode::Ok
}

pub async fn kernel_entry(r: DisplayResources) -> ! {
    let mut display_spi_config = spi::Config::default();
    display_spi_config.polarity = spi::Polarity::IdleLow;
    display_spi_config.phase = spi::Phase::CaptureOnFirstTransition;
    display_spi_config.frequency = 600000;
    let display_spi = Spi::new_txonly(r.spi, r.sck, r.mosi, r.dma, Irqs, display_spi_config);
    let display_spi_cs = gpio::Output::new(r.cs, gpio::Level::Low);
    let mut display = ST7920::new(display_spi, display_spi_cs, false);
    let mut delay = Delay {};
    display.init(&mut delay).await;
    display.flush(&mut delay).await;

    defmt::debug!("signal core1 to launch shell process");
    APP_LAUNCH_SIG.signal(get_shell_app_entry());
    cortex_m::asm::sev();

    let font = IBM437_8X8_REGULAR;
    let text_style = MonoTextStyle::new(&font, BinaryColor::On);
    let background_style = PrimitiveStyle::with_fill(BinaryColor::Off);

    loop {
        defmt::debug!("waiting for next display flush");
        FLUSH_DISPLAY_SIG.wait().await;
        // XXX assumes always in text-mode
        // draw zero over whole canvas
        Rectangle::new(Point::zero(), Size::new(128, 64))
            .into_styled(background_style)
            .draw(&mut display)
            .unwrap();
        // draw line-by-line
        let mut point = Point::new(0, font.character_size.height as i32);
        let n_lines = 64 / font.character_size.height as usize;
        let line_length = 128 / font.character_size.width as usize;
        for line_i in 0..n_lines {
            let line;
            unsafe {
                line = &DISPLAY_FB[line_i * line_length..(line_i + 1) * line_length];
            }
            defmt::debug!("{:?}", line);
            let text_line = str::from_utf8(line).unwrap().trim_end_matches("\0");
            Text::with_alignment(text_line, point, text_style, Alignment::Left)
                .draw(&mut display)
                .unwrap();
            point += Size::new(0, text_style.line_height());
        }
        display.flush(&mut delay).await;
        defmt::debug!("done display flush");
    }
}

async fn usb_entry(r: UsbResources) {
    let driver = embassy_rp::usb::host::Driver::new(r.usb, Irqs);
    static BUS_STATE: embassy_usb_host::BusState = embassy_usb_host::BusState::new();
    let (mut bus_ctrl, bus) = embassy_usb_host::bus(driver, &BUS_STATE);

    defmt::debug!("USB host initialized, waiting for device...");

    loop {
        let speed = bus_ctrl.wait_for_connection().await;
        defmt::debug!("Device connected at speed {:?}", speed);

        let mut config_buf = [0u8; 256];
        let result = bus
            .enumerate(embassy_usb_host::BusRoute::Direct(speed), &mut config_buf)
            .await;

        let (enum_info, config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                defmt::error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        defmt::debug!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            enum_info.device_desc.vendor_id,
            enum_info.device_desc.product_id,
            enum_info.device_address
        );

        let mut hid = match embassy_usb_host::class::hid::HidHost::new(
            &bus,
            &config_buf[..config_len],
            &enum_info,
        ) {
            Ok(h) => h,
            Err(e) => {
                defmt::error!("HID init failed: {:?}", e);
                continue;
            }
        };

        if let Err(e) = hid.set_idle(0, 0).await {
            defmt::error!("SET_IDLE failed: {:?}", e);
            continue;
        }

        defmt::debug!("HID device ready, load keyboard driver");
        let mut kbd = drivers::Keyboard::setup(&mut hid, KEYBOARD_EVENT_CHANNEL.dyn_sender()).await;
        kbd.entry().await;
    }
}

/// main task loop for handling user processes.
///
/// Should be called once on core1, will block indefinitely.
fn user_process_supervisor(abi: *const KernelAbi) -> ! {
    #[allow(clippy::never_loop)]
    loop {
        if let Some(app_entry) = APP_LAUNCH_SIG.try_take() {
            defmt::debug!("core1 received new app entry, launching...");
            APP_EXIT_SIG.reset();
            let exit_code = app_entry(abi);
            defmt::info!(
                "core1 process finished, got exit code '{}'",
                exit_code as u8
            );
            APP_EXIT_SIG.signal(exit_code);
            defmt::debug!("parking core1, until new app is launched");
        }
        cortex_m::asm::wfe();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::debug!("loader entry");
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);
    embassy_rp::multicore::spawn_core1(
        p.CORE1,
        // TODO replace addr_of_mut with newer implementation
        unsafe { &mut *addr_of_mut!(APP_STACK) },
        move || user_process_supervisor(&KERNEL_ABI as *const KernelAbi),
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(core0_task(r).unwrap()))
}

#[embassy_executor::task]
async fn core0_task(r: AssignedResources) {
    defmt::debug!("kernel entry");
    join(kernel_entry(r.display), usb_entry(r.usb)).await;
}
