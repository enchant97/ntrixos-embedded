use core::{ffi::c_void, ptr::NonNull};

use sdk::{
    FileDescriptor,
    errno::{self, KernelResult},
    syscall::{RMapSyscall, RSyncSyscall, RUnmapSyscall},
};

use crate::{
    core::abi,
    syscall::{req_r_map, req_r_sync, req_r_unmap},
};

pub type RefDesc = isize;

pub struct FileDesc {
    descriptor: FileDescriptor,
}

impl FileDesc {
    #[must_use]
    pub fn from_fd(fd: FileDescriptor) -> Self {
        Self { descriptor: fd }
    }

    pub fn write(&self, buf: &[u8]) {
        (abi().write)(self.descriptor, buf.as_ptr(), buf.len());
    }

    pub fn read<const N: usize>(&self, buf: &mut [u8; N]) -> KernelResult<usize> {
        let out_read = (abi().read)(self.descriptor, buf.as_mut_ptr(), buf.len());
        if out_read < 0 {
            return Err(errno::GENERAL);
        }
        Ok(out_read as usize)
    }

    /// Read directly from kernel to pointer.
    ///
    /// Can be used when descriptor returns a different data-type than `[u8; N]`.
    ///
    /// # Safety
    /// - Must provide a valid memory address that matches the size of `buf_len`
    /// - Buffer pointer must not be null
    pub unsafe fn read_ptr(&self, buf: NonNull<u8>, buf_len: usize) -> KernelResult<usize> {
        let out_read = (abi().read)(self.descriptor, buf.as_ptr(), buf_len);
        if out_read < 0 {
            return Err(errno::GENERAL);
        }
        Ok(out_read as usize)
    }

    pub fn flush(&self) {
        (abi().flush)(self.descriptor);
    }

    pub fn r_map(&mut self, addr: NonNull<u8>, len: usize) -> KernelResult<RefDesc> {
        let desc = req_r_map(RMapSyscall {
            addr: addr.as_ptr(),
            len,
            fd: self.descriptor,
        });
        if desc >= 0 { Ok(desc) } else { Err(desc) }
    }

    pub fn r_sync(&mut self, ref_desc: RefDesc) -> KernelResult<()> {
        let code = req_r_sync(RSyncSyscall { desc: ref_desc });
        if code >= 0 { Ok(()) } else { Err(code) }
    }

    pub fn r_unmap(&mut self, ref_desc: RefDesc) -> KernelResult<()> {
        let code = req_r_unmap(RUnmapSyscall { desc: ref_desc });
        if code >= 0 { Ok(()) } else { Err(code) }
    }

    pub fn ioctl(
        &self,
        op: usize,
        in_arg: *const c_void,
        out_arg: *mut c_void,
    ) -> KernelResult<()> {
        let code = (abi().ioctl)(self.descriptor, op, in_arg, out_arg);
        if code == errno::OK { Ok(()) } else { Err(code) }
    }
}
