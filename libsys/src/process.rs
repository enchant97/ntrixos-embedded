use sdk::errno::{self, ErrNo};

use crate::core::abi;

/// Exit the program with given exit code.
pub fn exit(code: ErrNo) -> ! {
    (abi().exit)(code)
}
/// Exit the program with generic error code.
pub fn abort() -> ! {
    (abi().exit)(errno::GENERAL)
}
