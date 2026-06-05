//! Commonly used misc functions and types
use sdk::{KernelAbi, errno::ErrNo};

/// A runnable user app entry function.
///
/// Either points to a location in user apps flash or user memory.
pub type AppEntry = extern "C" fn(*const KernelAbi) -> ErrNo;
