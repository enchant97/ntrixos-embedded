use core::sync::atomic::Ordering;

use num_enum::TryFromPrimitive;
use sdk::{
    errno::ErrNo,
    syscall::{SyscallNum, syscall_message},
};

pub fn zero_syscall_data() {
    let msg = &syscall_message();
    msg.num.store(0, Ordering::Relaxed);
    msg.result.store(isize::MIN, Ordering::Relaxed);
    for arg in &msg.args {
        arg.store(0, Ordering::Relaxed);
    }
}

pub fn unpack_num() -> Result<SyscallNum, usize> {
    let num = syscall_message().num.load(Ordering::Acquire);
    SyscallNum::try_from_primitive(num).map_err(|_| num)
}

pub fn unpack_exit_args() -> ErrNo {
    let arg0 = syscall_message().args[0].load(Ordering::Relaxed);
    arg0 as isize
}
