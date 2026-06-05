use core::{
    cell::{RefCell, RefMut},
    mem::MaybeUninit,
};

use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use sdk::{
    FileDescriptor,
    drivers::keyboard::KeyEvent,
    errno::{self, KernelResult},
};

use crate::fd::FileDesc;

pub struct KeyboardRaw {
    _private: (),
}

pub struct Keyboard {
    inner: Mutex<ThreadModeRawMutex, RefCell<KeyboardRaw>>,
}

impl Keyboard {
    pub fn lock<U>(&self, f: impl FnOnce(&mut RefMut<'_, KeyboardRaw>) -> U) -> U {
        unsafe { self.inner.lock_mut(|v| f(&mut v.borrow_mut())) }
    }
}

impl KeyboardRaw {
    pub fn read_key_blocking(&self) -> KernelResult<KeyEvent> {
        let mut event = MaybeUninit::<KeyEvent>::uninit();
        let n = unsafe {
            FileDesc::from_fd(FileDescriptor::KeyEvents)
                .read_ptr(event.as_mut_ptr() as *mut u8, size_of::<KeyEvent>())?
        };
        if n != size_of::<KeyEvent>() {
            Err(errno::GENERAL)
        } else {
            Ok(unsafe { event.assume_init() })
        }
    }
}

static KEYBOARD: Keyboard = Keyboard {
    inner: Mutex::new(RefCell::new(KeyboardRaw { _private: () })),
};

#[must_use]
pub fn keyboard() -> &'static Keyboard {
    &KEYBOARD
}
