//! A terminal handler for a display
#![allow(clippy::too_many_arguments)]
use crate::{
    display::{CharacterDisplay, DisplayRaw, PixelDisplay},
    sdk::drivers::display::{CharAttributes, CharCell, DisplayMode},
};
use heapless::Vec;

type PromptPrefx = [CharCell; 2];
const PROMPT_PREFIX: PromptPrefx = [CharCell::from_u8_lossy(b'?'), CharCell::from_u8_lossy(b'>')];
const PROMPT_WRAP_PREFIX: PromptPrefx =
    [CharCell::from_u8_lossy(b'-'), CharCell::from_u8_lossy(b'>')];

#[derive(Debug, Clone, Copy)]
pub struct TerminalCursor {
    pos: usize,
    enable: bool,
}

impl TerminalCursor {
    pub const fn enable(&mut self) {
        self.pos = 0;
        self.enable = true;
    }

    pub const fn disable(&mut self) {
        self.enable = false;
    }

    pub const fn move_right(&mut self, v: usize) {
        self.pos = self.pos.saturating_add(v);
    }

    pub const fn move_left(&mut self, v: usize) {
        self.pos = self.pos.saturating_sub(v);
    }

    pub const fn pos(&self) -> usize {
        self.pos
    }

    pub const fn enabled(&self) -> bool {
        self.enable
    }
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self {
            pos: 0,
            enable: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TerminalStat {
    rows: usize,
    cols: usize,
}

pub struct Terminal<'a, const PROMPT_BUFFER: usize> {
    current_row: usize,
    display: &'a mut DisplayRaw,
    term_stat: TerminalStat,
    prompt_buffer: Vec<u8, PROMPT_BUFFER>,
    prompt_start_row: Option<usize>,
    prompt_cursor: TerminalCursor,
}

impl<'a, const PROMPT_BUFFER: usize> Terminal<'a, PROMPT_BUFFER> {
    pub fn setup(display: &'a mut DisplayRaw) -> Self {
        display
            .set_display_mode(DisplayMode::Character)
            .expect("failed to set mode");
        let display_stat = display.get_display_stat().expect("failed to get stat");
        let mut term = Self {
            current_row: 0,
            display,
            term_stat: TerminalStat {
                rows: display_stat.char_rows as usize,
                cols: display_stat.char_cols as usize,
            },
            prompt_buffer: Vec::new(),
            prompt_start_row: None,
            prompt_cursor: Default::default(),
        };
        term.clear();
        term
    }

    /// Zero display and reset terminal state.
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
            draw_line(
                fb,
                self.term_stat,
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
        self.prompt_cursor.enable();
        self.prompt_start_row = Some(self.current_row);
        self.display.with_charcell_buffer_flushed(|fb| {
            redraw_prompt(
                fb,
                self.term_stat,
                &PROMPT_PREFIX,
                &PROMPT_WRAP_PREFIX,
                &mut self.current_row,
                &mut self.prompt_start_row,
                self.prompt_cursor,
                &self.prompt_buffer,
            );
        });
    }

    pub fn feed_prompt(&mut self, glyph: u8) -> Result<(), u8> {
        self.prompt_buffer.push(glyph)?;
        self.prompt_cursor.move_right(1);
        self.display.with_charcell_buffer_flushed(|fb| {
            redraw_prompt(
                fb,
                self.term_stat,
                &PROMPT_PREFIX,
                &PROMPT_WRAP_PREFIX,
                &mut self.current_row,
                &mut self.prompt_start_row,
                self.prompt_cursor,
                &self.prompt_buffer,
            );
        });
        Ok(())
    }

    pub fn feed_prompt_backspace(&mut self) {
        if self.prompt_buffer.pop().is_some() {
            self.prompt_cursor.move_left(1);
            self.display.with_charcell_buffer_flushed(|fb| {
                redraw_prompt(
                    fb,
                    self.term_stat,
                    &PROMPT_PREFIX,
                    &PROMPT_WRAP_PREFIX,
                    &mut self.current_row,
                    &mut self.prompt_start_row,
                    self.prompt_cursor,
                    &self.prompt_buffer,
                );
            });
        }
    }

    pub fn end_prompt(&mut self) -> Vec<u8, PROMPT_BUFFER> {
        self.prompt_cursor.disable();
        self.display.with_charcell_buffer_flushed(|fb| {
            redraw_prompt(
                fb,
                self.term_stat,
                &PROMPT_PREFIX,
                &PROMPT_WRAP_PREFIX,
                &mut self.current_row,
                &mut self.prompt_start_row,
                self.prompt_cursor,
                &self.prompt_buffer,
            );
        });
        let out = self.prompt_buffer.clone();
        self.prompt_buffer.clear();
        self.prompt_start_row = None;
        out
    }
}

const fn calc_row_start(stat: TerminalStat, row: usize) -> usize {
    row * stat.cols
}

fn shift_up(
    fb: &mut [CharCell],
    stat: TerminalStat,
    current_row: &mut usize,
    prompt_start_row: &mut Option<usize>,
) {
    // shift rows up
    for i in 0..stat.rows - 1 {
        let src_row_start = calc_row_start(stat, i + 1);
        fb.copy_within(
            src_row_start..src_row_start + stat.cols,
            calc_row_start(stat, i),
        );
    }
    // zero last row
    let row_start = calc_row_start(stat, stat.rows - 1);
    fb[row_start..row_start + stat.cols].fill(CharCell::from_u8_lossy(b' '));
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
    stat: TerminalStat,
    line: &[CharCell],
    prefix: Option<&[CharCell]>,
    wrap_prefix: Option<&[CharCell]>,
    current_row: &mut usize,
    prompt_start_row: &mut Option<usize>,
) {
    let usable = stat.cols - prefix.map(|v| v.len()).unwrap_or(0);
    let num_chunks = line.len().div_ceil(usable).max(1);

    // shift until there's room for all chunks
    while *current_row + num_chunks > stat.rows {
        shift_up(fb, stat, current_row, prompt_start_row);
    }

    // write each chunk, one character at a time
    for i in 0..num_chunks {
        let pfx = if i == 0 { prefix } else { wrap_prefix };
        let chunk_start = i * usable;
        let mut col = 0;
        if let Some(pfx) = pfx {
            for ch in pfx {
                fb[calc_row_start(stat, *current_row) + col] = *ch;
                col += 1;
            }
        }
        for ch in line[chunk_start..(chunk_start + usable).min(line.len())].iter() {
            fb[calc_row_start(stat, *current_row) + col] = *ch;
            col += 1;
        }
        while col < stat.cols {
            fb[calc_row_start(stat, *current_row) + col] = CharCell::from_u8_lossy(b' ');
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
    stat: TerminalStat,
    prefix: &[CharCell],
    wrap_prefix: &[CharCell],
    current_row: &mut usize,
    prompt_start_row: &mut Option<usize>,
    prompt_cursor: TerminalCursor,
    prompt_buffer: &[u8],
) {
    let usable = stat.cols - prefix.len();
    // +1 to allocate enough space for the cursor
    let num_chunks = (prompt_buffer.len() + 1).div_ceil(usable).max(1);

    // shift until there's room for all chunks
    while prompt_start_row.unwrap() + num_chunks > stat.rows {
        shift_up(fb, stat, current_row, prompt_start_row);
    }

    // overwrite prompt from start row
    *current_row = prompt_start_row.unwrap();
    for i in 0..num_chunks {
        let pfx = if i == 0 { prefix } else { wrap_prefix };
        let text_start = i * usable;
        let mut col = 0;
        for ch in pfx {
            fb[calc_row_start(stat, *current_row) + col] = *ch;
            col += 1;
        }
        for ch in prompt_buffer[text_start..(text_start + usable).min(prompt_buffer.len())].iter() {
            fb[calc_row_start(stat, *current_row) + col] = CharCell::from_u8_lossy(*ch);
            col += 1;
        }
        while col < stat.cols {
            fb[calc_row_start(stat, *current_row) + col] = CharCell::from_u8_lossy(b' ');
            col += 1;
        }
        // draw/hide current cursor
        if (text_start..=text_start + usable).contains(&prompt_cursor.pos()) {
            let cursor_col = prefix.len() + (prompt_cursor.pos() - text_start);
            fb[calc_row_start(stat, *current_row) + cursor_col]
                .attrs
                .set(CharAttributes::INVERT, prompt_cursor.enabled());
        }
        *current_row += 1;
    }

    // clear any rows that belonged to a prev longer prompt
    while *current_row < stat.rows
        && !are_cells_blank(
            &fb[calc_row_start(stat, *current_row)..calc_row_start(stat, *current_row) + stat.cols],
        )
    {
        let src_row_start = calc_row_start(stat, *current_row);
        fb[src_row_start..src_row_start + stat.cols].fill(CharCell::from_u8_lossy(b' '));
        *current_row += 1;
    }

    // reset current row back to end of prompt
    *current_row = prompt_start_row.unwrap() + num_chunks;
}
