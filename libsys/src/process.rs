use sdk::errno::{self, ErrNo};

use crate::syscall;

/// Exit the program with given exit code.
pub fn exit(code: ErrNo) -> ! {
    syscall::req_exit(code)
}
/// Exit the program with generic error code.
#[inline(always)]
pub fn abort() -> ! {
    exit(errno::GENERAL)
}
