#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use assign_resources::assign_resources;
use core::cell::RefCell;
use core::ptr::{NonNull, addr_of_mut};
use core::slice::from_raw_parts;
use core::sync::atomic::Ordering;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Executor;
use embassy_futures::join::join;
use embassy_rp::interrupt;
use embassy_rp::peripherals::{DMA_CH1, SPI0, USB};
use embassy_rp::spinlock_mutex::blocking_mutex::SpinlockMutex;
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
use portable_atomic::{AtomicBool, AtomicU32};
use sdk::drivers::display::{
    CharCell, DisplayCharOperation, DisplayCharStat, DisplayOperation, DisplayStat,
};
use sdk::errno::{self, ErrNo};
use sdk::syscall::SyscallNum;
use sdk::{FileDescriptor, KernelAbi, kcom};
use static_cell::StaticCell;
use tryslab::Slab;
use tryslab::heapless::DequeSlab;

use crate::common::{AppEntry, RawPtr, RdTableEntry};
use crate::drivers::display::{DISPLAY_CHAR_STAT, DISPLAY_STAT, DisplayDriver};
use crate::memory::get_shell_app_entry;
use crate::signaling::{
    k_signal_user_restart, k_signal_user_restart_reset, k_signal_user_restart_setup,
};

mod common;
mod drivers;
mod memory;
mod signaling;
mod syscall;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use embassy_rp::gpio;
    use embedded_hal::delay::DelayNs;
    // guard against recursive panic
    static KERNEL_PANICKING: AtomicBool = AtomicBool::new(false);
    if KERNEL_PANICKING.swap(true, Ordering::SeqCst) {
        #[cfg(feature = "debug-panic")]
        cortex_m::asm::udf();
        #[cfg(not(feature = "debug-panic"))]
        loop {
            cortex_m::asm::nop();
        }
    } else {
        cortex_m::interrupt::disable();
        // forcefully stop core1 using hardware register
        embassy_rp::pac::PSM
            .frce_off()
            .modify(|w| w.set_proc1(true));
        // always output full panic info over rtt
        defmt::error!("{}", defmt::Display2Format(info));
        #[cfg(feature = "debug-panic")]
        {
            cortex_m::asm::udf();
        }
        #[cfg(not(feature = "debug-panic"))]
        {
            // this status led only works on non-w variants
            let led_pin = unsafe { embassy_rp::peripherals::PIN_25::steal() };
            let mut led = gpio::Output::new(led_pin, gpio::Level::Low);
            let mut delay = embassy_time::Delay;
            loop {
                led.toggle();
                delay.delay_ms(1000);
            }
        }
    }
}

static mut APP_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();

static KCOM_REQ: Signal<CriticalSectionRawMutex, kcom::KComType> = Signal::new();

#[interrupt]
unsafe fn SIO_IRQ_PROC0() {
    let sio = embassy_rp::pac::SIO;
    sio.fifo().st().write(|w| w.set_wof(false)); // ack overflow flag
    while sio.fifo().st().read().vld() {
        let kcom_type = unsafe { kcom::read_kcom_fifo_unchecked().unwrap() };
        KCOM_REQ.signal(kcom_type);
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn TIMER_IRQ_3() {
    // perform exception return, by building frame.
    core::arch::naked_asm!(
        "mov r4, lr",
        "mov r0, sp",
        "bl {handler}",
        "mov sp, r0",
        "bx r4",
        handler = sym build_restart_frame,
    )
}

#[unsafe(no_mangle)]
extern "C" fn build_restart_frame(_old_frame: u32) -> u32 {
    k_signal_user_restart_reset();

    let code = -1; // TODO get real exit code
    APP_EXIT_SIG.signal(code);
    if !APP_LAUNCH_SIG.signaled() {
        // TODO this should be a kernel task
        // TODO memory is not reset
        APP_LAUNCH_SIG.signal(get_shell_app_entry());
    }

    let supervisor_sp = APP_SUPERVISOR_SP.load(Ordering::Acquire);
    let frame_base = (supervisor_sp - 32) as *mut u32;
    unsafe {
        core::ptr::write_volatile(frame_base.add(0), 0); // r0
        core::ptr::write_volatile(frame_base.add(1), 0); // r1
        core::ptr::write_volatile(frame_base.add(2), 0); // r2
        core::ptr::write_volatile(frame_base.add(3), 0); // r3
        core::ptr::write_volatile(frame_base.add(4), 0); // r12
        core::ptr::write_volatile(frame_base.add(5), 0); // LR (unused, supervisor never "returns")
        core::ptr::write_volatile(frame_base.add(6), user_process_supervisor as u32); // return PC
        core::ptr::write_volatile(frame_base.add(7), 0x0100_0000); // xPSR, T-bit set
    }
    frame_base as u32
}

pub static KERNEL_ABI: KernelAbi = KernelAbi {};

/// Used to get back to the app supervisor to launch/relaunch app.
static APP_SUPERVISOR_SP: AtomicU32 = AtomicU32::new(0);
/// Used to signal that the current app has finished.
pub static APP_EXIT_SIG: Signal<CriticalSectionRawMutex, ErrNo> = Signal::new();
/// Used to signal which app to launch.
pub static APP_LAUNCH_SIG: Signal<CriticalSectionRawMutex, AppEntry> = Signal::new();

static SPI0_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Spi<SPI0, spi::Async>>> =
    StaticCell::new();
static FLUSH_CHAR_DISPLAY_SIG: Signal<CriticalSectionRawMutex, RdTableEntry> = Signal::new();
static FLUSH_CHAR_DISPLAY_DONE_SIG: Signal<CriticalSectionRawMutex, ()> = Signal::new();
const RD_TABLE_SPINLOCK_N: usize = 1;
static RD_TABLE: SpinlockMutex<RD_TABLE_SPINLOCK_N, RefCell<DequeSlab<RdTableEntry, 4>>> =
    SpinlockMutex::new(RefCell::new(DequeSlab::new()));

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

fn syscall_exit() {
    // TODO use exit code
    let code = syscall::unpack_exit_args();
    k_signal_user_restart();
}

fn syscall_read() {
    let req = syscall::unpack_read();
    let result = match req.fd {
        FileDescriptor::Display => -1,
        FileDescriptor::DisplayChar => -1,
        FileDescriptor::KeyEvents => {
            use sdk::drivers::keyboard::KeyEvent;
            if req.len < size_of::<KeyEvent>() {
                // buffer too small
                -1
            } else {
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
                    (req.buf as *mut KeyEvent).write(event);
                }
                size_of::<KeyEvent>() as isize
            }
        }
    };
    syscall::pack_response(SyscallNum::Read, result);
    kcom::write_kcom_fifo_blocking(kcom::KComType::Syscall);
}

fn syscall_r_map() {
    let req = syscall::unpack_r_map();
    let result = match req.fd {
        FileDescriptor::DisplayChar => RD_TABLE.lock(|rd_table| {
            let mut rd_table = rd_table.borrow_mut();
            let raw_ptr = if let Some(addr) = NonNull::new(req.addr) {
                RawPtr(addr)
            } else {
                return -1;
            };
            let ref_desc = rd_table
                .try_insert(RdTableEntry {
                    raw_ptr,
                    len: req.len,
                    fd: req.fd,
                })
                .unwrap();
            ref_desc as isize
        }),
        _ => -1,
    };
    syscall::pack_response(SyscallNum::RMap, result);
    kcom::write_kcom_fifo_blocking(kcom::KComType::Syscall);
}

fn syscall_r_unmap() {
    let req = syscall::unpack_r_unmap();
    let result = RD_TABLE.lock(|rd_table| {
        let mut rd_table = rd_table.borrow_mut();
        rd_table
            .try_remove(req.desc as usize)
            .map(|_| 0)
            .unwrap_or(-1)
    });
    syscall::pack_response(SyscallNum::RUnmap, result);
    kcom::write_kcom_fifo_blocking(kcom::KComType::Syscall);
}

fn syscall_r_sync() {
    let req = syscall::unpack_r_sync();
    let result = RD_TABLE.lock(|rd_table| {
        let rd_table = rd_table.borrow_mut();
        let rd_ref = *rd_table.get(req.desc as usize).unwrap();
        match rd_ref.fd {
            FileDescriptor::DisplayChar => {
                FLUSH_CHAR_DISPLAY_SIG.signal(rd_ref);
                while !FLUSH_CHAR_DISPLAY_DONE_SIG.signaled() {
                    cortex_m::asm::nop();
                }
                FLUSH_CHAR_DISPLAY_DONE_SIG.reset();
                0
            }
            _ => -1,
        }
    });
    syscall::pack_response(SyscallNum::RSync, result);
    kcom::write_kcom_fifo_blocking(kcom::KComType::Syscall);
}

fn syscall_ioctl() {
    let req = syscall::unpack_ioctl();
    match req.fd {
        FileDescriptor::Display => {
            let disp_op = DisplayOperation::try_from(req.op).unwrap();
            match disp_op {
                DisplayOperation::GetStat => unsafe {
                    *(req.out_arg as *mut DisplayStat) = DISPLAY_STAT;
                },
            }
        }
        FileDescriptor::DisplayChar => {
            let disp_op = DisplayCharOperation::try_from(req.op).unwrap();
            match disp_op {
                DisplayCharOperation::GetStat => unsafe {
                    *(req.out_arg as *mut DisplayCharStat) = DISPLAY_CHAR_STAT;
                },
            }
        }
        _ => todo!(),
    }
    syscall::pack_response(SyscallNum::IoCtl, errno::OK);
    kcom::write_kcom_fifo_blocking(kcom::KComType::Syscall);
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
    let mut display = DisplayDriver::init(display_spi, display_busy).await;
    defmt::debug!("display ready");

    defmt::debug!("signal core1 to launch shell process");
    APP_LAUNCH_SIG.signal(get_shell_app_entry());
    cortex_m::asm::sev();

    let mut display_loop = async || {
        loop {
            defmt::debug!("waiting for next display flush");
            let rd_ref = FLUSH_CHAR_DISPLAY_SIG.wait().await;
            let buff: &[CharCell] =
                unsafe { from_raw_parts(rd_ref.raw_ptr.as_ptr() as *const CharCell, rd_ref.len) };
            display.flush_char(buff).await;
            FLUSH_CHAR_DISPLAY_DONE_SIG.signal(());
            defmt::debug!("done display flush");
        }
    };
    let kcom_loop = async || {
        loop {
            // XXX just assume syscall, since it's the only request
            let _ = KCOM_REQ.wait().await;
            let syscall_num = syscall::unpack_num().expect("unexpected syscall number");
            match syscall_num {
                SyscallNum::Null => {
                    defmt::debug!("got syscall NULL, ignoring");
                }
                SyscallNum::Exit => syscall_exit(),
                SyscallNum::Read => syscall_read(),
                SyscallNum::RMap => syscall_r_map(),
                SyscallNum::RUnmap => syscall_r_unmap(),
                SyscallNum::RSync => syscall_r_sync(),
                SyscallNum::IoCtl => syscall_ioctl(),
            }
        }
    };
    join(display_loop(), kcom_loop()).await;
    unreachable!()
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
        RD_TABLE.lock(|rd| rd.borrow_mut().clear()); // TODO should be a kernel task
        syscall::zero_syscall_data(); // TODO should be a kernel task

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
        move || {
            use embassy_rp::interrupt;
            use embassy_rp::interrupt::InterruptExt;
            // disable embassy interrupts on core1,
            // required for kernel message parsing.
            interrupt::SIO_IRQ_PROC1.disable();
            // TODO: replace with below when switching to RP235x
            //embassy_rp::interrupt::SIO_IRQ_FIFO.disable();

            k_signal_user_restart_setup();

            user_process_supervisor()
        },
    );
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| spawner.spawn(core0_task(r).unwrap()))
}

#[embassy_executor::task]
async fn core0_task(r: AssignedResources) {
    use embassy_rp::interrupt;
    use embassy_rp::interrupt::InterruptExt;
    // NOTE I think this could be accomplished in bind_interrupts?
    unsafe { interrupt::SIO_IRQ_PROC0.enable() };
    defmt::debug!("kernel entry");
    join(kernel_entry(r.display), usb_entry(r.usb)).await;
}
