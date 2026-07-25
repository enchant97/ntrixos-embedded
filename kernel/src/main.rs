#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use assign_resources::assign_resources;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::Ordering;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Executor;
use embassy_futures::join::join;
use embassy_rp::peripherals::{DMA_CH1, SPI0, USB};
use embassy_rp::{
    Peri, bind_interrupts, gpio,
    multicore::Stack,
    peripherals::{self, DMA_CH0},
    spi::{self, Spi},
};
use embassy_sync::mutex::Mutex;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use portable_atomic::AtomicU32;
use sdk::drivers::display::{DisplayMode, DisplayOperation, DisplayStat};
use sdk::errno::{self, ErrNo};
use sdk::{FileDescriptor, KernelAbi};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::common::AppEntry;
use crate::drivers::display::{DISPLAY_STAT, DisplayDriver};
use crate::memory::get_shell_app_entry;

mod common;
mod drivers;
mod memory;

static mut APP_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

pub static KERNEL_ABI: KernelAbi = KernelAbi {
    get_version: abi_get_version,
    exit: abi_exit,
    write: abi_write,
    read: abi_read,
    flush: abi_flush,
    seek: abi_seek,
    mmap: abi_mmap,
    ioctl: abi_ioctl,
};

/// Used to get back to the app supervisor to launch/relaunch app.
static APP_SUPERVISOR_SP: AtomicU32 = AtomicU32::new(0);
/// Used to signal that the current app has finished.
pub static APP_EXIT_SIG: Signal<CriticalSectionRawMutex, ErrNo> = Signal::new();
/// Used to signal which app to launch.
pub static APP_LAUNCH_SIG: Signal<CriticalSectionRawMutex, AppEntry> = Signal::new();

static SPI0_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<SPI0, spi::Async>>> =
    StaticCell::new();
static FLUSH_DISPLAY_SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static mut DISPLAY_PIXEL_BUFFER: *mut u8 = null_mut();
static mut DISPLAY_CHAR_BUFFER: *mut u8 = null_mut();

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
        miso: PIN_16,
        dma_rx: DMA_CH0,
        dma_tx: DMA_CH1,
        busy: PIN_20,
    },
    usb: UsbResources {
        usb: USB,
    },
}

bind_interrupts!(struct Irqs{
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

extern "C" fn abi_get_version() -> u32 {
    1
}

extern "C" fn abi_exit(code: ErrNo) -> ! {
    APP_EXIT_SIG.signal(code);
    if !APP_LAUNCH_SIG.signaled() {
        // TODO this should be a kernel task
        // TODO memory is not reset
        APP_LAUNCH_SIG.signal(get_shell_app_entry());
    }
    let sp = APP_SUPERVISOR_SP.load(Ordering::Acquire);
    unsafe {
        core::arch::asm!(
        "mov sp, {sp}",
        "bx {resume}",
        sp=in(reg) sp,
        resume = in(reg) user_process_supervisor as u32 | 1,
        options(noreturn)
        );
    }
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
        FileDescriptor::Display => unsafe {
            // HACK assumes character-mode
            DISPLAY_CHAR_BUFFER as *mut c_void
        },
        FileDescriptor::KeyEvents => null_mut(),
    }
}

extern "C" fn abi_ioctl(
    fd: FileDescriptor,
    op: usize,
    in_arg: *const c_void,
    out_arg: *mut c_void,
) -> ErrNo {
    match fd {
        FileDescriptor::Display => {
            let disp_op = DisplayOperation::try_from(op).unwrap();
            match disp_op {
                DisplayOperation::GetMode => unsafe {
                    *(out_arg as *mut DisplayMode) = DisplayMode::Character;
                },
                DisplayOperation::GetStat => unsafe {
                    *(out_arg as *mut DisplayStat) = DISPLAY_STAT;
                },
                DisplayOperation::SetModeCharacter => {}
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
    errno::OK
}

pub async fn kernel_entry(r: DisplayResources) -> ! {
    defmt::debug!("main kernel entry");

    let spi0_bus = SPI0_BUS.init_with(|| {
        Mutex::new(Spi::new(
            r.spi,
            r.sck,
            r.mosi,
            r.miso,
            r.dma_tx,
            r.dma_rx,
            Irqs,
            Default::default(),
        ))
    });

    defmt::debug!("setting up display");
    let mut display_spi_config = spi::Config::default();
    display_spi_config.polarity = spi::Polarity::IdleLow;
    display_spi_config.phase = spi::Phase::CaptureOnSecondTransition;
    let display_spi_cs = gpio::Output::new(r.cs, gpio::Level::Low);
    let display_spi = SpiDeviceWithConfig::new(spi0_bus, display_spi_cs, display_spi_config);
    let display_busy = gpio::Input::new(r.busy, gpio::Pull::Up);
    let mut display = DisplayDriver::new(display_spi, display_busy);
    display.init().await;
    unsafe {
        DISPLAY_PIXEL_BUFFER = display.pixel_buffer_as_mut_ptr();
        DISPLAY_CHAR_BUFFER = display.char_buffer_as_mut_ptr();
    }
    defmt::debug!("display ready");

    defmt::debug!("signal core1 to launch shell process");
    APP_LAUNCH_SIG.signal(get_shell_app_entry());
    cortex_m::asm::sev();

    loop {
        defmt::debug!("waiting for next display flush");
        FLUSH_DISPLAY_SIG.wait().await;
        display.flush().await;
        defmt::debug!("done display flush");
    }
}

async fn usb_entry(r: UsbResources) {
    defmt::debug!("usb entry");
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
fn user_process_supervisor() -> ! {
    let sp: u32;
    unsafe {
        core::arch::asm!("mov {sp}, sp", sp = out(reg) sp,);
    }
    APP_SUPERVISOR_SP.store(sp, Ordering::Release);

    if let Some(exit_code) = APP_EXIT_SIG.try_take() {
        defmt::debug!("core1 process exit early with code '{}'", exit_code);
    }

    defmt::debug!("starting user process supervisor");
    loop {
        if let Some(app_entry) = APP_LAUNCH_SIG.try_take() {
            defmt::debug!("core1 received new app entry, launching...");
            APP_EXIT_SIG.reset();
            let exit_code = app_entry(&KERNEL_ABI as *const KernelAbi);
            defmt::info!("core1 process finished, got exit code '{}'", exit_code);
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
        move || user_process_supervisor(),
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(core0_task(r).unwrap()))
}

#[embassy_executor::task]
async fn core0_task(r: AssignedResources) {
    defmt::debug!("kernel entry");
    join(kernel_entry(r.display), usb_entry(r.usb)).await;
}
