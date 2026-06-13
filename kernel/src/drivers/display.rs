use crate::drivers::display::custom::BUFFER_SIZE;
pub use custom::CustomDisplay;
use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};
use sdk::drivers::display::{CharCell, DisplayStat};
use static_cell::StaticCell;

mod custom;

pub const PIXEL_HEIGHT: usize = custom::HEIGHT;
pub const PIXEL_WIDTH: usize = custom::WIDTH;
pub const FONT_HEIGHT: usize = 8;
pub const FONT_WIDTH: usize = 8;
pub const CHAR_ROWS: usize = PIXEL_HEIGHT / FONT_HEIGHT as usize;
pub const CHAR_COLS: usize = PIXEL_WIDTH / FONT_WIDTH as usize;
pub const CHAR_BUFFER_SIZE: usize = CHAR_ROWS * CHAR_COLS;
pub const DISPLAY_STAT: DisplayStat = DisplayStat {
    pixel_width: PIXEL_WIDTH as u32,
    pixel_height: PIXEL_HEIGHT as u32,
    char_rows: CHAR_ROWS as u32,
    char_cols: CHAR_COLS as u32,
};

static PIXEL_BUFFER: StaticCell<[u8; BUFFER_SIZE]> = StaticCell::new();
static CHARACTER_BUFFER: StaticCell<[CharCell; CHAR_BUFFER_SIZE]> = StaticCell::new();

fn render_char_cell(fb: &mut [u8], col: usize, row: usize, cell: &CharCell) {
    const FB_STRIDE: usize = (CHAR_COLS * FONT_WIDTH) / 8;
    const SHEET_STRIDE: usize = ibm437::CHARS_PER_ROW;
    let offset = cell.glyph as usize;
    let glyph_col = offset % ibm437::CHARS_PER_ROW;
    let glyph_row = offset / ibm437::CHARS_PER_ROW;
    for y in 0..FONT_HEIGHT {
        let sheet_byte = ibm437::IBM437_8X8_REGULAR_DATA
            [(glyph_row * FONT_WIDTH + y) * SHEET_STRIDE + glyph_col];
        let byte = if cell.attrs.contains_invert() {
            !sheet_byte
        } else {
            sheet_byte
        };
        fb[(row * FONT_HEIGHT + y) * FB_STRIDE + col] = byte;
    }
}

pub struct DisplayDriver<SPI> {
    raw_display: CustomDisplay<SPI>,
    pixel_buffer: &'static mut [u8; BUFFER_SIZE],
    char_buffer: &'static mut [CharCell; CHAR_BUFFER_SIZE],
}

impl<SPI> DisplayDriver<SPI>
where
    SPI: SpiDevice,
{
    pub const fn pixel_width(&self) -> usize {
        PIXEL_WIDTH
    }

    pub const fn pixel_height(&self) -> usize {
        PIXEL_HEIGHT
    }

    pub const fn char_rows(&self) -> usize {
        CHAR_ROWS
    }

    pub const fn char_columns(&self) -> usize {
        CHAR_COLS
    }

    pub fn new(raw_display: CustomDisplay<SPI>) -> Self {
        Self {
            raw_display,
            pixel_buffer: PIXEL_BUFFER.init_with(|| [0u8; BUFFER_SIZE]),
            char_buffer: CHARACTER_BUFFER
                .init_with(|| [CharCell::from_u8_lossy(0); CHAR_BUFFER_SIZE]),
        }
    }

    pub unsafe fn pixel_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.pixel_buffer.as_mut_ptr()
    }

    pub unsafe fn char_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.char_buffer.as_mut_ptr() as *mut u8
    }

    /// Render the character buffer onto the pixel buffer
    pub fn render_chars(&mut self) {
        // draw line-by-line
        for line_i in 0..self.char_rows() {
            let cells =
                &self.char_buffer[line_i * self.char_columns()..(line_i + 1) * self.char_columns()];
            for (col_i, cell) in cells.iter().enumerate() {
                render_char_cell(self.pixel_buffer, col_i, line_i, cell);
            }
        }
    }

    /// Flush the software buffer onto the physical display
    pub async fn flush(&mut self, delay: &mut impl DelayNs) {
        self.raw_display.update(delay, self.pixel_buffer).await;
    }

    /// Force clear the physical display
    pub async fn clear(&mut self, delay: &mut impl DelayNs) {
        self.raw_display.clear(delay).await;
    }
}
