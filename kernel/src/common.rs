//! Commonly used misc functions and types
use core::ptr::NonNull;

use sdk::{FileDescriptor, KernelAbi, errno::ErrNo};

/// A runnable user app entry function.
///
/// Either points to a location in user apps flash or user memory.
pub type AppEntry = extern "C" fn(*const KernelAbi) -> ErrNo;

#[derive(Debug, Clone, Copy)]
pub struct RawPtr(pub NonNull<u8>);
unsafe impl Send for RawPtr {}
unsafe impl Sync for RawPtr {}

impl RawPtr {
    pub fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RdTableEntry {
    pub raw_ptr: RawPtr,
    pub len: usize,
    pub fd: FileDescriptor,
}
