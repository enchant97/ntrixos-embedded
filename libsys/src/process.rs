pub use sdk::ExitCode;

use crate::core::abi;

/// Exit the program with given exit code.
pub fn exit(code: ExitCode) -> ! {
    (abi().exit)(code)
}
/// Exit the program with generic error code.
pub fn abort() -> ! {
    (abi().exit)(ExitCode::GeneralError)
}
