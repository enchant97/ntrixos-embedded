use core::{ffi::c_void, sync::atomic::Ordering};

use num_enum::TryFromPrimitive;
use sdk::{
    FileDescriptor,
    errno::ErrNo,
    syscall::{
        IoCtlSyscall, RMapSyscall, RSyncSyscall, RUnmapSyscall, ReadSyscall, SyscallNum,
        syscall_message,
    },
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

pub fn unpack_read() -> ReadSyscall {
    let msg = syscall_message();
    ReadSyscall {
        fd: FileDescriptor::try_from_primitive(msg.args[0].load(Ordering::Relaxed)).unwrap(),
        buf: msg.args[1].load(Ordering::Relaxed) as *mut u8,
        len: msg.args[2].load(Ordering::Relaxed),
    }
}

pub fn unpack_r_map() -> RMapSyscall {
    let msg = syscall_message();
    RMapSyscall {
        addr: msg.args[0].load(Ordering::Relaxed) as *mut u8,
        len: msg.args[1].load(Ordering::Relaxed),
        fd: FileDescriptor::try_from_primitive(msg.args[2].load(Ordering::Relaxed)).unwrap(),
    }
}

pub fn unpack_r_sync() -> RSyncSyscall {
    let msg = syscall_message();
    RSyncSyscall {
        desc: msg.args[0].load(Ordering::Relaxed) as isize,
    }
}

pub fn unpack_r_unmap() -> RUnmapSyscall {
    let msg = syscall_message();
    RUnmapSyscall {
        desc: msg.args[0].load(Ordering::Relaxed) as isize,
    }
}

pub fn unpack_ioctl() -> IoCtlSyscall {
    let msg = syscall_message();
    IoCtlSyscall {
        fd: FileDescriptor::try_from_primitive(msg.args[0].load(Ordering::Relaxed)).unwrap(),
        op: msg.args[1].load(Ordering::Relaxed),
        in_arg: msg.args[2].load(Ordering::Relaxed) as *const c_void,
        out_arg: msg.args[3].load(Ordering::Relaxed) as *mut c_void,
    }
}

pub fn pack_response(num: SyscallNum, result: isize) {
    let msg = syscall_message();
    msg.result.store(result, Ordering::Relaxed);
    msg.num.store(num as usize, Ordering::Release);
}
