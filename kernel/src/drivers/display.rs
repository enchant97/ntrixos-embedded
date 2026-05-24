use embedded_graphics::{
    Drawable,
    framebuffer::{Framebuffer, buffer_size},
    geometry::{Point, Size},
    mono_font::MonoTextStyle,
    pixelcolor::{
        BinaryColor,
        raw::{LittleEndian, RawU1},
    },
    text::{
        Alignment, Text,
        renderer::{CharacterStyle, TextRenderer},
    },
};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::{delay::DelayNs, spi::SpiBus};

mod st7920;

use ibm437::IBM437_8X8_REGULAR;
use sdk::drivers::display::DisplayStat;
pub use st7920::ST7920;

use crate::drivers::display::st7920::{HEIGHT, WIDTH};

pub const PIXEL_HEIGHT: usize = HEIGHT;
pub const PIXEL_WIDTH: usize = WIDTH;
pub const CHAR_ROWS: usize = HEIGHT / IBM437_8X8_REGULAR.character_size.height as usize;
pub const CHAR_COLS: usize = WIDTH / IBM437_8X8_REGULAR.character_size.width as usize;
pub const CHAR_BUFFER_SIZE: usize = CHAR_ROWS * CHAR_COLS;
pub const DISPLAY_STAT: DisplayStat = DisplayStat {
    pixel_width: PIXEL_WIDTH as u32,
    pixel_height: PIXEL_HEIGHT as u32,
    char_rows: CHAR_ROWS as u32,
    char_cols: CHAR_COLS as u32,
};

pub struct DisplayDriver<SPI, CS> {
    raw_display: ST7920<SPI, CS>,
    frame_buffer: Framebuffer<
        BinaryColor,
        RawU1,
        LittleEndian,
        WIDTH,
        HEIGHT,
        { buffer_size::<BinaryColor>(WIDTH, HEIGHT) },
    >,
    char_buffer: [u8; CHAR_BUFFER_SIZE],
}

impl<SPI, CS> DisplayDriver<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
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

    pub fn new(raw_display: ST7920<SPI, CS>) -> Self {
        Self {
            raw_display,
            frame_buffer: Framebuffer::new(),
            char_buffer: [0; CHAR_BUFFER_SIZE],
        }
    }

    pub unsafe fn pixel_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.frame_buffer.data_mut().as_mut_ptr()
    }

    pub unsafe fn char_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.char_buffer.as_mut_ptr()
    }

    /// Render the character buffer onto the pixel buffer
    pub fn render_chars(&mut self) {
        let font = IBM437_8X8_REGULAR;
        let mut text_style = MonoTextStyle::new(&font, BinaryColor::On);
        text_style.set_background_color(Some(BinaryColor::Off));
        // draw line-by-line
        let mut point = Point::new(0, font.character_size.height as i32);
        for line_i in 0..self.char_rows() {
            let line =
                &self.char_buffer[line_i * self.char_columns()..(line_i + 1) * self.char_columns()];
            let text_line = str::from_utf8(line).unwrap().trim_end_matches("\0");
            Text::with_alignment(text_line, point, text_style, Alignment::Left)
                .draw(&mut self.frame_buffer)
                .unwrap();
            point += Size::new(0, text_style.line_height());
        }
    }

    /// Flush the software buffer onto the physical display
    pub async fn flush(&mut self, delay: &mut impl DelayNs) {
        self.raw_display
            .update(delay, self.frame_buffer.data())
            .await;
    }
}
