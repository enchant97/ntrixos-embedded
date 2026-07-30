use num_enum::TryFromPrimitive;

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
    pub num: SyscallNum,
    pub args: [usize; 6],
    pub result: isize,
}
