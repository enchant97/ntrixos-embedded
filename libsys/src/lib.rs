#![no_std]

pub mod char_display;
pub mod display;
pub mod fd;
pub mod keyboard;
pub mod mem;
pub mod process;
mod syscall;
pub mod terminal;

pub use libsys_macros::{char_cells, entrypoint};
/// Re-Export of SDK
pub use sdk;

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
    use crate::process::exit;
    use core::panic::PanicInfo;
    use sdk::errno;

    #[panic_handler]
    fn panic(_: &PanicInfo) -> ! {
        exit(errno::GENERAL);
    }
}
