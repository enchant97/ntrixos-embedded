#![no_std]
#![no_main]

mod terminal;

use libsys::{
    char_cells, entrypoint,
    process::exit,
    sdk::{
        drivers::keyboard::{Action, KeyKind},
        errno,
    },
};

use terminal::Terminal;

#[entrypoint]
fn main() {
    libsys::display::display().lock(|d| {
        libsys::keyboard::keyboard().lock(|kb| {
            let mut term = Terminal::<8, 16, 256>::setup(d);
            term.feed_output_line(&char_cells!("Welcome,"));
            term.feed_output_line(&char_cells!("To The OS!"));
            term.start_prompt();
            loop {
                let key_event = kb.read_key_blocking().unwrap();
                if key_event.action == Action::Press && key_event.kind == KeyKind::Char {
                    let _ = term.feed_prompt(key_event.code);
                } else if key_event.action == Action::Press && key_event.code == 0x2a {
                    term.feed_prompt_backspace();
                } else if key_event.action == Action::Press && key_event.code == 0x28 {
                    let out = term.end_prompt();
                    if out.len() == 0 {
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
