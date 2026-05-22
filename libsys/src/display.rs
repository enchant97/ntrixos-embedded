use crate::fd::FileDesc;
use core::{
    cell::{RefCell, RefMut},
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{null, null_mut},
};
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use nostd::io::Write;
use sdk::drivers::display::{DisplayMode, DisplayOperation, DisplayStat};
use sdk::{ExitCode, FileDescriptor};

pub struct DisplayRaw {
    _private: (),
}

pub struct Display {
    inner: Mutex<ThreadModeRawMutex, RefCell<DisplayRaw>>,
}

impl Display {
    pub fn lock<U>(&self, f: impl FnOnce(&mut RefMut<'_, DisplayRaw>) -> U) -> U {
        unsafe { self.inner.lock_mut(|v| f(&mut v.borrow_mut())) }
    }
}

impl DisplayRaw {
    pub fn get_display_mode(&self) -> Result<DisplayMode, ExitCode> {
        let mut display_mode = DisplayMode::Character;
        FileDesc::from_fd(FileDescriptor::Display)
            .ioctl(
                DisplayOperation::GetMode as usize,
                null(),
                &raw mut display_mode as *mut c_void,
            )
            .map(|()| display_mode)
    }

    pub fn set_display_mode(&mut self, display_mode: DisplayMode) -> Result<(), ExitCode> {
        FileDesc::from_fd(FileDescriptor::Display).ioctl(
            match display_mode {
                DisplayMode::Pixel => DisplayOperation::SetModePixel as usize,
                DisplayMode::Character => DisplayOperation::SetModeCharacter as usize,
            },
            &raw const display_mode as *mut c_void,
            null_mut(),
        )
    }

    pub fn get_display_stat(&self) -> Result<DisplayStat, ExitCode> {
        let mut display_stat = MaybeUninit::uninit();
        FileDesc::from_fd(FileDescriptor::Display)
            .ioctl(
                DisplayOperation::GetStat as usize,
                null(),
                &raw mut display_stat as *mut c_void,
            )
            .map(|()| unsafe { display_stat.assume_init() })
    }

    pub fn get_framebuffer_mut(&mut self, op: impl FnOnce(&mut [u8])) {
        op(FileDesc::from_fd(FileDescriptor::Display)
            .mmap(16 * 8) // HACK get actual size from stat & current mode
            .unwrap())
    }
}

static DISPLAY: Display = Display {
    inner: Mutex::new(RefCell::new(DisplayRaw { _private: () })),
};

#[must_use]
pub fn display() -> &'static Display {
    &DISPLAY
}

impl Write for DisplayRaw {
    fn write(&mut self, buf: &[u8]) -> nostd::io::Result<usize> {
        FileDesc::from_fd(FileDescriptor::Display).write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> nostd::io::Result<()> {
        FileDesc::from_fd(FileDescriptor::Display).flush();
        Ok(())
    }
}
