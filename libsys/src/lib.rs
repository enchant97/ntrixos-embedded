#![no_std]

pub mod alloc;
pub mod core;
pub mod display;
pub mod fd;
pub mod process;

pub use process::ExitCode;
pub use sdk::KernelAbi; // remove when proc-macro is implemented for _start

/// Re-export used parts of nostd.
pub mod nostd {
    pub mod io {
        pub use nostd::io::{Read, Write};
    }
}

/// Custom panic handler for capturing Rust panics.
///
/// Will send relevant exit code to kernel.
#[cfg(target_os = "none")]
pub mod panic_system {
    use crate::{ExitCode, process::exit};
    use core::panic::PanicInfo;

    #[panic_handler]
    fn panic(_: &PanicInfo) -> ! {
        exit(ExitCode::GeneralError);
    }
}
