use core::sync::atomic::Ordering;

use sdk::{
    errno::ErrNo,
    kcom::{KComType, read_kcom_fifo_blocking, write_kcom_fifo_blocking},
    syscall::{RMapSyscall, RSyncSyscall, RUnmapSyscall, SyscallNum, syscall_message},
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
