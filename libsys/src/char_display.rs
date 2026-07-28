use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{NonNull, null},
};
use portable_atomic::{AtomicBool, Ordering};

use crate::fd::{FileDesc, RefDesc};
use crate::sdk::{
    FileDescriptor,
    drivers::display::{CharCell, DisplayCharOperation, DisplayCharStat},
    errno::KernelResult,
};

static DISPLAY_CHAR_EXISTS: AtomicBool = AtomicBool::new(false);

pub struct CharDisplayBuffer<'a> {
    ref_desc: RefDesc,
    inner: &'a mut [CharCell],
}

impl<'a> CharDisplayBuffer<'a> {
    pub fn sync(&mut self) -> KernelResult<()> {
        FileDesc::from_fd(FileDescriptor::DisplayChar).r_sync(self.ref_desc)
    }
}

impl<'a> Drop for CharDisplayBuffer<'a> {
    fn drop(&mut self) {
        FileDesc::from_fd(FileDescriptor::DisplayChar)
            .r_unmap(self.ref_desc)
            .expect("failed to r_unmap display char buffer");
    }
}

impl<'a> CharDisplayBuffer<'a> {
    fn init(buf: &'a mut [CharCell]) -> KernelResult<Self> {
        let ref_desc = FileDesc::from_fd(FileDescriptor::DisplayChar).r_map(
            unsafe { NonNull::new_unchecked(buf.as_mut_ptr() as *mut u8) },
            buf.len(),
        )?;
        Ok(Self {
            ref_desc,
            inner: buf,
        })
    }

    pub fn buffer(&'a self) -> &'a [CharCell] {
        self.inner
    }

    pub fn buffer_mut(&'a mut self) -> &'a mut [CharCell] {
        self.inner
    }

    #[inline(always)]
    pub fn with_buffer_flushed(&mut self, op: impl FnOnce(&mut [CharCell])) -> KernelResult<()> {
        op(self.inner);
        self.sync()
    }
}

pub struct CharDisplay {
    _private: (),
}

impl CharDisplay {
    pub fn init() -> Option<Self> {
        if DISPLAY_CHAR_EXISTS
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            None
        } else {
            Some(Self { _private: () })
        }
    }

    pub fn get_stat() -> KernelResult<DisplayCharStat> {
        let mut stat = MaybeUninit::uninit();
        FileDesc::from_fd(FileDescriptor::DisplayChar)
            .ioctl(
                DisplayCharOperation::GetStat as usize,
                null(),
                &raw mut stat as *mut c_void,
            )
            .map(|()| unsafe { stat.assume_init() })
    }

    pub fn register_buffer<'a>(
        &self,
        buf: &'a mut [CharCell],
    ) -> KernelResult<CharDisplayBuffer<'a>> {
        CharDisplayBuffer::init(buf)
    }
}

impl Drop for CharDisplay {
    fn drop(&mut self) {
        DISPLAY_CHAR_EXISTS.store(false, Ordering::Release);
    }
}
