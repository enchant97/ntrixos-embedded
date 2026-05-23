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
        let mut display_mode = MaybeUninit::uninit();
        FileDesc::from_fd(FileDescriptor::Display)
            .ioctl(
                DisplayOperation::GetMode as usize,
                null(),
                &raw mut display_mode as *mut c_void,
            )
            .map(|()| unsafe { display_mode.assume_init() })
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

    fn buffer_mut(&mut self) -> &mut [u8] {
        // PERF getting mode+stat each time is costly
        let mode = self.get_display_mode().unwrap();
        let stat = self.get_display_stat().unwrap();
        let buffer_size = match mode {
            DisplayMode::Pixel => (stat.pixel_width / 8) * stat.pixel_height,
            DisplayMode::Character => stat.char_rows * stat.char_cols,
        };
        FileDesc::from_fd(FileDescriptor::Display)
            .mmap(buffer_size as usize)
            .unwrap()
    }

    /// Get a mutable buffer.
    #[inline]
    pub fn with_buffer(&mut self, op: impl FnOnce(&mut [u8])) {
        op(self.buffer_mut());
    }

    /// Get a mutable buffer and flush once operation is complete.
    #[inline]
    pub fn with_buffer_flushed(&mut self, op: impl FnOnce(&mut [u8])) {
        self.with_buffer(op);
        self.flush().expect("failed to flush");
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
