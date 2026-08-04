#![no_std]

use core::ffi::c_void;
use num_enum::TryFromPrimitive;

pub mod drivers;
pub mod errno;
pub mod kcom;
pub mod syscall;

#[repr(usize)]
#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
pub enum FileDescriptor {
    KeyEvents,
    Display,
    DisplayChar,
}

#[repr(C)]
pub struct KernelAbi {
    /// Write directly to the given file descriptor.
    pub write: extern "C" fn(fd: FileDescriptor, buff: *const u8, buff_len: usize),
    /// Read directly from given file descriptor.
    pub read: extern "C" fn(fd: FileDescriptor, buff: *mut u8, buff_len: usize) -> isize,
    /// Ensure everything that is buffered is written to given descriptor.
    pub flush: extern "C" fn(fd: FileDescriptor),
    /// Adjust current cursor of given file descriptor.
    ///
    /// Only one cursor exists per file descriptor.
    pub seek: extern "C" fn(fd: FileDescriptor, offset: usize),
    /// Device Control
    ///
    /// Each type of device will have a different set of available commands and argument values.
    pub ioctl: extern "C" fn(
        fd: FileDescriptor,
        op: usize,
        in_arg: *const c_void,
        out_arg: *mut c_void,
    ) -> isize,
}
