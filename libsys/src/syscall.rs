use core::sync::atomic::Ordering;

use sdk::{
    errno::ErrNo,
    kcom::{KComType, write_kcom_fifo_blocking},
    syscall::{SyscallNum, syscall_message},
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
