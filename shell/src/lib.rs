#![no_std]

use libsys::{
    char_cells, entrypoint,
    process::exit,
    sdk::{
        drivers::keyboard::{Action, KeyKind},
        errno,
    },
    terminal::Terminal,
};

#[entrypoint]
pub fn app_entry() {
    libsys::display::display().lock(|d| {
        libsys::keyboard::keyboard().lock(|kb| {
            let mut term = Terminal::<256>::setup(d);
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
    });
}
