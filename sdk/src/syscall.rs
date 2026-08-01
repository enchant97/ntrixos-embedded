use num_enum::TryFromPrimitive;
use portable_atomic::{AtomicIsize, AtomicUsize};

#[unsafe(link_section = ".syscall_message")]
pub static SYSCALL_MESSAGE: Syscall = Syscall::empty();

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
