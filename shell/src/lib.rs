#![no_std]
#![allow(static_mut_refs)]

use libsys::{
    char_cells,
    char_display::CharDisplay,
    entrypoint,
    mem::write_kcom_fifo_blocking,
    process::exit,
    sdk::{
        drivers::{
            display::{CharCell, DisplayCharStat},
            keyboard::{Action, KeyKind},
        },
        errno,
    },
    terminal::Terminal,
};

static mut CHAR_BUFFER: [CharCell; 15 * 20] = [CharCell::from_u8_lossy(0); 15 * 20];

#[entrypoint]
pub fn app_entry() {
    write_kcom_fifo_blocking(libsys::sdk::kcom::KComType::Syscall);

    libsys::keyboard::keyboard().lock(|kb| {
        let char_display = CharDisplay::init().unwrap();
        let mut char_buffer = unsafe { char_display.register_buffer(&mut CHAR_BUFFER).unwrap() };
        let mut term =
            Terminal::<256>::setup(&mut char_buffer, &DisplayCharStat { rows: 15, cols: 20 });
        term.feed_output_line(&char_cells!("Welcome, To Ntrix OS!"));
        term.start_prompt();
        loop {
            let key_event = kb.read_key_blocking().unwrap();
            if key_event.action == Action::Press && key_event.kind == KeyKind::Char {
                let _ = term.feed_prompt(key_event.code);
            } else if key_event.action == Action::Press && key_event.code == 0x2a {
                term.feed_prompt_backspace();
            } else if key_event.action == Action::Press && key_event.code == 0x28 {
                let out = term.end_prompt();
                if out.is_empty() {
                    term.start_prompt();
                } else {
                    term.feed_output_line(&char_cells!("Done"));
                    term.start_prompt();
                }
            } else if key_event.action == Action::Press && key_event.code == 0x29 {
                exit(errno::OK);
            }
        }
    });
}
