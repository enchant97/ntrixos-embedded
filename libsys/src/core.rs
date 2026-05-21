use sdk::KernelAbi;

static mut ABI_PTR: *const KernelAbi = core::ptr::null();

/// Init the system library.
///
/// Must be called once on program start.
pub fn sys_init(abi: *const KernelAbi) {
    unsafe {
        if !ABI_PTR.is_null() {
            debug_assert!(false, "sys_init called multiple times");
            return; // skip overwriting pointer
        }
        ABI_PTR = abi;
    }
}

pub(crate) fn abi() -> &'static KernelAbi {
    unsafe {
        assert!(!ABI_PTR.is_null());
        &*ABI_PTR
    }
}

/// Get the current version of the kernel abi.
#[must_use]
pub fn get_abi_version() -> u32 {
    (abi().get_version)()
}
