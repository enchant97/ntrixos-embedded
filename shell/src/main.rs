#![no_std]
#![no_main]

use libsys::{
    char_cells,
    display::CharWriter,
    entrypoint,
    process::exit,
    sdk::drivers::{
        display::{CharAttributes, CharCell, DisplayMode},
        keyboard::{Action, KeyEvent, KeyKind},
    },
};

#[entrypoint]
fn main() {
    libsys::display::display().lock(|d| {
        d.set_display_mode(DisplayMode::Character)
            .expect("failed to set mode");
        d.with_buffer_flushed(|mut fb| {
            fb.fill(0);
            let msg = char_cells!("Hello World!");
            fb.write_all_cells(&msg).unwrap();
        });
    });

    libsys::keyboard::keyboard().lock(|kb| {
        libsys::display::display().lock(|d| {
            let mut key_event: KeyEvent;
            loop {
                key_event = kb.read_key_blocking().unwrap();
                if key_event.action == Action::Press && key_event.kind == KeyKind::Char {
                    d.with_buffer_flushed(|mut fb| {
                        let msg = char_cells!("Key Pressed = ");
                        let mut pressed_char = CharCell::from_u8_lossy(key_event.code);
                        pressed_char.attrs.insert(CharAttributes::INVERT);
                        fb.fill(0);
                        fb.write_all_cells(&msg).unwrap();
                        fb.write_all_cells(&[pressed_char]).unwrap();
                    });
                } else if key_event.code == 0x29 {
                    exit(libsys::sdk::ExitCode::Ok);
                }
            }
        });
    });
}
