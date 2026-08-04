#![no_std]

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
pub struct KernelAbi {}
