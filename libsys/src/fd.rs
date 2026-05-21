use core::{ffi::c_void, slice::from_raw_parts_mut};

use sdk::{ExitCode, FileDescriptor};

use crate::core::abi;

pub struct FileDesc {
    descriptor: FileDescriptor,
}

impl FileDesc {
    #[must_use]
    pub fn from_fd(fd: FileDescriptor) -> Self {
        Self { descriptor: fd }
    }

    pub fn write(&self, buf: &[u8]) {
        (abi().write)(self.descriptor, buf.as_ptr(), buf.len());
    }

    pub fn flush(&self) {
        (abi().flush)(self.descriptor);
    }

    pub fn mmap(&mut self, len: usize) -> Option<&mut [u8]> {
        let ptr = (abi().mmap)(self.descriptor);
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(from_raw_parts_mut(ptr as *mut u8, len)) }
        }
    }

    pub fn ioctl(
        &self,
        op: usize,
        in_arg: *const c_void,
        out_arg: *mut c_void,
    ) -> Result<(), ExitCode> {
        let code = (abi().ioctl)(self.descriptor, op, in_arg, out_arg);
        if code == ExitCode::Ok {
            Ok(())
        } else {
            Err(code)
        }
    }
}
