#![no_std]

use core::ffi::c_void;

pub mod display;

/// Used to report the exit code of the program.
#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExitCode {
    /// Success
    Ok = 0,
    /// Generic error, use a specific one if available
    GeneralError = 1,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FileDescriptor {
    Display,
}

#[repr(C)]
pub struct KernelAbi {
    pub get_version: extern "C" fn() -> u32,
    pub exit: extern "C" fn(ExitCode) -> !,
    /// Memory allocation request
    pub malloc: extern "C" fn(size: usize) -> *mut u8,
    /// Memory removal request, pointer must have been previously returned from `malloc()`.
    pub free: extern "C" fn(ptr: *mut u8),
    /// Write directly to the given file descriptor.
    pub write: extern "C" fn(fd: FileDescriptor, buff: *const u8, buff_len: usize),
    /// Ensure everything that is buffered is written to given descriptor.
    pub flush: extern "C" fn(fd: FileDescriptor),
    /// Adjust current cursor of given file descriptor.
    ///
    /// Only one cursor exists per file descriptor.
    pub seek: extern "C" fn(fd: FileDescriptor, offset: usize),
    /// Get a shared buffer for given file descriptor.
    ///
    /// - Requires `flush(fd)` to ensure changes get written.
    pub mmap: extern "C" fn(fd: FileDescriptor) -> *mut c_void,
    /// Device Control
    ///
    /// Each type of device will have a different set of available commands and argument values.
    pub ioctl: extern "C" fn(
        fd: FileDescriptor,
        op: usize,
        in_arg: *const c_void,
        out_arg: *mut c_void,
    ) -> ExitCode,
}
