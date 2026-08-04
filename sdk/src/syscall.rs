use core::{
    ffi::c_void,
    sync::atomic::{AtomicIsize, AtomicUsize},
};
use num_enum::TryFromPrimitive;

use crate::FileDescriptor;

unsafe extern "C" {
    static SYSCALL_MESSAGE: Syscall;
}

/// Current syscall message stored in shared memory.
///
/// Should be validated for using.
pub fn syscall_message() -> &'static Syscall {
    unsafe { &SYSCALL_MESSAGE }
}

#[derive(Debug, PartialEq, Clone, Copy, TryFromPrimitive)]
#[repr(usize)]
pub enum SyscallNum {
    Null = 0,
    /// Request early program exit
    Exit,
    //Write,
    /// Read directly from given fd
    Read,
    //Flush,
    //Seek,
    /// Register a user memory mapping to a fd
    RMap,
    /// Un-register a user memory mapping
    RUnmap,
    /// Sync a user memory mapping with fd
    RSync,
    /// Manipulate device parameters of a given fd,
    /// expected inputs/outputs vary based on device/driver.
    IoCtl,
}

#[derive(Debug)]
#[repr(C)]
pub struct Syscall {
    pub num: AtomicUsize,
    pub args: [AtomicUsize; 6],
    pub result: AtomicIsize,
}

#[derive(Debug)]
#[repr(C)]
pub struct ReadSyscall {
    pub fd: FileDescriptor,
    pub buf: *mut u8,
    pub len: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct RMapSyscall {
    pub addr: *mut u8,
    pub len: usize,
    pub fd: FileDescriptor,
}

#[derive(Debug)]
#[repr(C)]
pub struct RSyncSyscall {
    pub desc: isize,
}

#[derive(Debug)]
#[repr(C)]
pub struct RUnmapSyscall {
    pub desc: isize,
}

#[derive(Debug)]
#[repr(C)]
pub struct IoCtlSyscall {
    pub fd: FileDescriptor,
    pub op: usize,
    pub in_arg: *const c_void,
    pub out_arg: *mut c_void,
}
