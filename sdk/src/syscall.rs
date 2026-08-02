use core::sync::atomic::{AtomicIsize, AtomicUsize};
use num_enum::TryFromPrimitive;

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
    Exit,
    Write,
    Read,
    Flush,
    Seek,
    RMap,
    RSync,
    RUnmap,
    IoCtl,
}

#[derive(Debug)]
#[repr(C)]
pub struct Syscall {
    pub num: AtomicUsize,
    pub args: [AtomicUsize; 6],
    pub result: AtomicIsize,
}
