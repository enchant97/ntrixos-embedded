#![no_std]
#![no_main]

use libsys::{
    entrypoint,
    nostd::io::Write,
    sdk::drivers::{
        display::DisplayMode,
        keyboard::{Action, KeyEvent, KeyKind},
    },
};

#[entrypoint]
fn main() {
    libsys::display::display().lock(|d| {
        d.set_display_mode(DisplayMode::Character)
            .expect("failed to set mode");
        d.with_buffer_flushed(|mut fb| {
            let msg = b"Hello World!";
            fb.write_all(msg).unwrap();
        });
    });

    libsys::keyboard::keyboard().lock(|kb| {
        libsys::display::display().lock(|d| {
            let mut key_event: KeyEvent;
            loop {
                key_event = kb.read_key_blocking().unwrap();
                if key_event.action == Action::Press && key_event.kind == KeyKind::Char {
                    d.with_buffer_flushed(|mut fb| {
                        let msg = format_args!("Key Pressed = {}", char::from(key_event.code));
                        fb.fill(0);
                        fb.write_fmt(msg).unwrap();
                    });
                } else if key_event.code == 0x29 {
                    break;
                }
            }
        });
    });
}
