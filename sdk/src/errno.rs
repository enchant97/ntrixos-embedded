pub type ErrNo = isize;
/// Success
pub const OK: ErrNo = 0;
/// Generic error, use a specific one if available
pub const GENERAL: ErrNo = -1;

pub type KernelResult<T> = Result<T, ErrNo>;
