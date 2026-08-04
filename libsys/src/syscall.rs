use core::sync::atomic::Ordering;

use sdk::{
    errno::ErrNo,
    kcom::{KComType, read_kcom_fifo_blocking, write_kcom_fifo_blocking},
    syscall::{
        IoCtlSyscall, RMapSyscall, RSyncSyscall, RUnmapSyscall, ReadSyscall, SyscallNum,
        syscall_message,
    },
};

pub fn req_exit(code: ErrNo) -> ! {
    let msg = &syscall_message();
    msg.args[0].store(code as usize, Ordering::Relaxed);
    msg.num.store(SyscallNum::Exit as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    // kernel cleans up after,
    // so just wait indefinitely
    loop {
        cortex_m::asm::wfe();
    }
}

pub fn req_r_read(req: ReadSyscall) -> isize {
    let msg = syscall_message();
    msg.args[0].store(req.fd as usize, Ordering::Relaxed);
    msg.args[1].store(req.buf as usize, Ordering::Relaxed);
    msg.args[2].store(req.len, Ordering::Relaxed);
    msg.num.store(SyscallNum::Read as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    let _ = read_kcom_fifo_blocking().unwrap();
    msg.result.load(Ordering::Acquire)
}

pub fn req_r_map(req: RMapSyscall) -> isize {
    let msg = syscall_message();
    msg.args[0].store(req.addr as usize, Ordering::Relaxed);
    msg.args[1].store(req.len, Ordering::Relaxed);
    msg.args[2].store(req.fd as usize, Ordering::Relaxed);
    msg.num.store(SyscallNum::RMap as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    let _ = read_kcom_fifo_blocking().unwrap();
    msg.result.load(Ordering::Acquire)
}

pub fn req_r_unmap(req: RUnmapSyscall) -> isize {
    let msg = syscall_message();
    msg.args[0].store(req.desc as usize, Ordering::Relaxed);
    msg.num
        .store(SyscallNum::RUnmap as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    let _ = read_kcom_fifo_blocking().unwrap();
    msg.result.load(Ordering::Acquire)
}

pub fn req_r_sync(req: RSyncSyscall) -> isize {
    let msg = syscall_message();
    msg.args[0].store(req.desc as usize, Ordering::Relaxed);
    msg.num.store(SyscallNum::RSync as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    let _ = read_kcom_fifo_blocking().unwrap();
    msg.result.load(Ordering::Acquire)
}

pub fn req_ioctl(req: IoCtlSyscall) -> isize {
    let msg = syscall_message();
    msg.args[0].store(req.fd as usize, Ordering::Relaxed);
    msg.args[1].store(req.op, Ordering::Relaxed);
    msg.args[2].store(req.in_arg as usize, Ordering::Relaxed);
    msg.args[3].store(req.out_arg as usize, Ordering::Relaxed);
    msg.num.store(SyscallNum::IoCtl as usize, Ordering::Release);
    write_kcom_fifo_blocking(KComType::Syscall);
    let _ = read_kcom_fifo_blocking().unwrap();
    msg.result.load(Ordering::Acquire)
}
