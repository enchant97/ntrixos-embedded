use core::sync::atomic::{AtomicIsize, AtomicUsize};
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
    Write,
    Read,
    Flush,
    Seek,
    /// Register a user memory mapping to a fd
    RMap,
    /// Un-register a user memory mapping
    RUnmap,
    /// Sync a user memory mapping with fd
    RSync,
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
