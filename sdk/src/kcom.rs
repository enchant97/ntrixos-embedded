use num_enum::TryFromPrimitive;
use portable_atomic::{AtomicIsize, AtomicUsize};

/// Type of Kernel Communication Message
#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
#[repr(u32)]
pub enum KComType {
    Syscall = 0,
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

impl Syscall {
    pub const fn empty() -> Self {
        Self {
            num: AtomicUsize::new(SyscallNum::Null as usize),
            args: [const { AtomicUsize::new(0) }; 6],
            result: AtomicIsize::new(isize::MIN),
        }
    }
}
