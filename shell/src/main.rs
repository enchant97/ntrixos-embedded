#![no_std]
#![no_main]

use heapless::Vec;
use libsys::{
    char_cells,
    display::DisplayRaw,
    entrypoint,
    process::exit,
    sdk::drivers::{
        display::{CharCell, DisplayMode},
        keyboard::{Action, KeyKind},
    },
};

type PromptPrefx = [CharCell; 2];
const PROMPT_PREFIX: PromptPrefx = char_cells!("?>");
const PROMPT_WRAP_PREFIX: PromptPrefx = char_cells!("->");

struct ShellScreen<'a, const MAX_ROWS: usize, const MAX_COLS: usize, const PROMPT_BUFFER: usize> {
    current_row: usize,
    display: &'a mut DisplayRaw,
    prompt_buffer: Vec<u8, PROMPT_BUFFER>,
    prompt_start_row: Option<usize>,
}

impl<'a, const MAX_ROWS: usize, const MAX_COLS: usize, const PROMPT_BUFFER: usize>
    ShellScreen<'a, MAX_ROWS, MAX_COLS, PROMPT_BUFFER>
{
    pub fn setup(display: &'a mut DisplayRaw) -> Self {
        display
            .set_display_mode(DisplayMode::Character)
            .expect("failed to set mode");
        let mut shell = Self {
            current_row: 0,
            display,
            prompt_buffer: Vec::new(),
            prompt_start_row: None,
        };
        shell.clear();
        shell
    }

    /// Zero display and reset shell state.
    pub fn clear(&mut self) {
        self.current_row = 0;
        self.prompt_buffer.clear();
        self.prompt_start_row = None;
        self.display.with_buffer_flushed(|fb| {
            fb.fill(0);
        });
    }

    pub fn feed_output_line(&mut self, cells: &[CharCell]) {
        self.display.with_charcell_buffer_flushed(|fb| {
            Self::draw_line(
                fb,
                cells,
                None,
                None,
                &mut self.current_row,
                &mut self.prompt_start_row,
            );
        });
    }

    pub fn start_prompt(&mut self) {
        self.prompt_buffer.clear();
        self.prompt_start_row = Some(self.current_row);
        self.display.with_charcell_buffer_flushed(|fb| {
            Self::redraw_prompt(
                fb,
                &PROMPT_PREFIX,
                &PROMPT_WRAP_PREFIX,
                &mut self.current_row,
                &mut self.prompt_start_row,
                &self.prompt_buffer,
            );
        });
    }

    pub fn feed_prompt(&mut self, glyph: u8) -> Result<(), u8> {
        self.prompt_buffer.push(glyph)?;
        self.display.with_charcell_buffer_flushed(|fb| {
            Self::redraw_prompt(
                fb,
                &PROMPT_PREFIX,
                &PROMPT_WRAP_PREFIX,
                &mut self.current_row,
                &mut self.prompt_start_row,
                &self.prompt_buffer,
            );
        });
        Ok(())
    }

    pub fn feed_prompt_backspace(&mut self) {
        if self.prompt_buffer.pop().is_some() {
            self.display.with_charcell_buffer_flushed(|fb| {
                Self::redraw_prompt(
                    fb,
                    &PROMPT_PREFIX,
                    &PROMPT_WRAP_PREFIX,
                    &mut self.current_row,
                    &mut self.prompt_start_row,
                    &self.prompt_buffer,
                );
            });
        }
    }

    pub fn end_prompt(&mut self) -> Vec<u8, PROMPT_BUFFER> {
        let out = self.prompt_buffer.clone();
        self.prompt_buffer.clear();
        self.prompt_start_row = None;
        out
    }

    const fn row_start(row: usize) -> usize {
        return row * MAX_COLS;
    }

    fn shift_up(
        fb: &mut [CharCell],
        current_row: &mut usize,
        prompt_start_row: &mut Option<usize>,
    ) {
        // shift rows up
        for i in 0..MAX_ROWS - 1 {
            let src_row_start = Self::row_start(i + 1);
            fb.copy_within(src_row_start..src_row_start + MAX_COLS, Self::row_start(i));
        }
        // zero last row
        let row_start = Self::row_start(MAX_ROWS - 1);
        fb[row_start..row_start + MAX_COLS].fill(CharCell::from_u8_lossy(b' '));
        // update indexes
        *current_row -= 1;
        if let Some(v) = prompt_start_row {
            *v -= 1;
        }
    }

    /// Draw a line, with wrapping.
    ///
    /// # Panics
    /// If `prefix` and `wrap_prefix` are not equal lengths.
    fn draw_line(
        fb: &mut [CharCell],
        line: &[CharCell],
        prefix: Option<&[CharCell]>,
        wrap_prefix: Option<&[CharCell]>,
        current_row: &mut usize,
        prompt_start_row: &mut Option<usize>,
    ) {
        let usable = MAX_COLS - prefix.map(|v| v.len()).unwrap_or(0);
        let num_chunks = line.len().div_ceil(usable).max(1);

        // shift until there's room for all chunks
        while *current_row + num_chunks > MAX_ROWS {
            Self::shift_up(fb, current_row, prompt_start_row);
        }

        // write each chunk, one character at a time
        for i in 0..num_chunks {
            let pfx = if i == 0 { prefix } else { wrap_prefix };
            let chunk_start = i * usable;
            let mut col = 0;
            if let Some(pfx) = pfx {
                for ch in pfx {
                    fb[Self::row_start(*current_row) + col] = *ch;
                    col += 1;
                }
            }
            for ch in line[chunk_start..(chunk_start + usable).min(line.len())].iter() {
                fb[Self::row_start(*current_row) + col] = *ch;
                col += 1;
            }
            while col < MAX_COLS {
                fb[Self::row_start(*current_row) + col] = CharCell::from_u8_lossy(b' ');
                col += 1;
            }
            *current_row += 1;
        }
    }

    fn are_cells_blank(cells: &[CharCell]) -> bool {
        for cell in cells {
            if *cell != CharCell::from_u8_lossy(b' ') && *cell != CharCell::from_u8_lossy(0) {
                return false;
            }
        }
        true
    }

    /// Draw the prompt
    ///
    /// # Panics
    /// - If `prefix` and `wrap_prefix` are not equal lengths.
    /// - `prompt_start_row` is `None`
    fn redraw_prompt(
        fb: &mut [CharCell],
        prefix: &[CharCell],
        wrap_prefix: &[CharCell],
        current_row: &mut usize,
        prompt_start_row: &mut Option<usize>,
        prompt_buffer: &[u8],
    ) {
        let usable = MAX_COLS - prefix.len();
        let num_chunks = prompt_buffer.len().div_ceil(usable).max(1);

        // shift until there's room for all chunks
        while prompt_start_row.unwrap() + num_chunks > MAX_ROWS {
            Self::shift_up(fb, current_row, prompt_start_row);
        }

        // overwrite prompt from start row
        *current_row = prompt_start_row.unwrap();
        for i in 0..num_chunks {
            let pfx = if i == 0 { prefix } else { wrap_prefix };
            let text_start = i * usable;
            let mut col = 0;
            for ch in pfx {
                fb[Self::row_start(*current_row) + col] = *ch;
                col += 1;
            }
            for ch in
                prompt_buffer[text_start..(text_start + usable).min(prompt_buffer.len())].iter()
            {
                fb[Self::row_start(*current_row) + col] = CharCell::from_u8_lossy(*ch);
                col += 1;
            }
            while col < MAX_COLS {
                fb[Self::row_start(*current_row) + col] = CharCell::from_u8_lossy(b' ');
                col += 1;
            }
            *current_row += 1;
        }

        // clear any rows that belonged to a prev longer prompt
        while *current_row < MAX_ROWS
            && !Self::are_cells_blank(
                &fb[Self::row_start(*current_row)..Self::row_start(*current_row) + MAX_COLS],
            )
        {
            let empty_row = [CharCell::from_u8_lossy(b' '); MAX_COLS];
            let src_row_start = Self::row_start(*current_row);
            fb[src_row_start..src_row_start + MAX_COLS].copy_from_slice(&empty_row);
            *current_row += 1;
        }

        // reset current row back to end of prompt
        *current_row = prompt_start_row.unwrap() + num_chunks;
    }
}

#[entrypoint]
fn main() {
    libsys::display::display().lock(|d| {
        libsys::keyboard::keyboard().lock(|kb| {
            let mut shell_screen = ShellScreen::<8, 16, 256>::setup(d);
            shell_screen.feed_output_line(&char_cells!("Welcome,"));
            shell_screen.feed_output_line(&char_cells!("To The OS!"));
            shell_screen.start_prompt();
            loop {
                let key_event = kb.read_key_blocking().unwrap();
                if key_event.action == Action::Press && key_event.kind == KeyKind::Char {
                    let _ = shell_screen.feed_prompt(key_event.code);
                } else if key_event.action == Action::Press && key_event.code == 0x2a {
                    shell_screen.feed_prompt_backspace();
                } else if key_event.action == Action::Press && key_event.code == 0x28 {
                    let out = shell_screen.end_prompt();
                    if out.len() == 0 {
                        shell_screen.start_prompt();
                    } else {
                        shell_screen.feed_output_line(&char_cells!("Done"));
                        shell_screen.start_prompt();
                    }
                } else if key_event.action == Action::Press && key_event.code == 0x29 {
                    exit(libsys::sdk::ExitCode::Ok);
                }
            }
        });
    });
}
