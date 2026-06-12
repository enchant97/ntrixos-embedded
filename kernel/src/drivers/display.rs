use embedded_graphics::{
    Drawable,
    framebuffer::{Framebuffer, buffer_size},
    geometry::{Point, Size},
    mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{
        BinaryColor,
        raw::{LittleEndian, RawU1},
    },
    text::{Alignment, Text, renderer::TextRenderer},
};
use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};

mod custom;

pub use custom::CustomDisplay;
use ibm437::IBM437_8X8_REGULAR;
use sdk::drivers::display::{CharAttributes, CharCell, DisplayStat};
use static_cell::StaticCell;

use crate::drivers::display::custom::{HEIGHT, WIDTH};

static DEFAULT_FONT: MonoFont = IBM437_8X8_REGULAR;
const DEFAULT_TEXT_STYLE: MonoTextStyle<BinaryColor> = build_text_style(CharAttributes::empty());

pub const PIXEL_HEIGHT: usize = HEIGHT;
pub const PIXEL_WIDTH: usize = WIDTH;
pub const CHAR_ROWS: usize = HEIGHT / DEFAULT_FONT.character_size.height as usize;
pub const CHAR_COLS: usize = WIDTH / DEFAULT_FONT.character_size.width as usize;
pub const CHAR_BUFFER_SIZE: usize = CHAR_ROWS * CHAR_COLS;
pub const DISPLAY_STAT: DisplayStat = DisplayStat {
    pixel_width: PIXEL_WIDTH as u32,
    pixel_height: PIXEL_HEIGHT as u32,
    char_rows: CHAR_ROWS as u32,
    char_cols: CHAR_COLS as u32,
};

type DisplayFrameBuffer = Framebuffer<
    BinaryColor,
    RawU1,
    LittleEndian,
    WIDTH,
    HEIGHT,
    { buffer_size::<BinaryColor>(WIDTH, HEIGHT) },
>;

static PIXEL_BUFFER: StaticCell<DisplayFrameBuffer> = StaticCell::new();
static CHARACTER_BUFFER: StaticCell<[CharCell; CHAR_BUFFER_SIZE]> = StaticCell::new();

const fn build_text_style<'a>(attrs: CharAttributes) -> MonoTextStyle<'a, BinaryColor> {
    let mut custom_style = MonoTextStyleBuilder::new().font(&DEFAULT_FONT);
    let fg_color = if attrs.contains_invert() {
        BinaryColor::Off
    } else {
        BinaryColor::On
    };
    custom_style = custom_style
        .text_color(fg_color)
        .background_color(fg_color.invert());
    if attrs.contains_underline() {
        custom_style = custom_style.underline()
    }
    if attrs.contains_strikethrough() {
        custom_style = custom_style.strikethrough();
    }
    custom_style.build()
}

pub struct DisplayDriver<SPI> {
    raw_display: CustomDisplay<SPI>,
    frame_buffer: &'static mut DisplayFrameBuffer,
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
            frame_buffer: PIXEL_BUFFER.init_with(|| Framebuffer::new()),
            char_buffer: CHARACTER_BUFFER
                .init_with(|| [CharCell::from_u8_lossy(0); CHAR_BUFFER_SIZE]),
        }
    }

    pub unsafe fn pixel_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.frame_buffer.data_mut().as_mut_ptr()
    }

    pub unsafe fn char_buffer_as_mut_ptr(&mut self) -> *mut u8 {
        self.char_buffer.as_mut_ptr() as *mut u8
    }

    /// Render the character buffer onto the pixel buffer
    pub fn render_chars(&mut self) {
        // draw line-by-line
        let mut point = Point::new(0, DEFAULT_FONT.character_size.height as i32);
        let line_height = DEFAULT_TEXT_STYLE.line_height();
        for line_i in 0..self.char_rows() {
            let cells =
                &self.char_buffer[line_i * self.char_columns()..(line_i + 1) * self.char_columns()];
            let mut char_point = point;
            for cell in cells {
                let text_style = if cell.attrs.is_empty() {
                    DEFAULT_TEXT_STYLE
                } else {
                    build_text_style(cell.attrs)
                };
                char_point =
                    Text::with_alignment(cell.as_str(), char_point, text_style, Alignment::Left)
                        .draw(self.frame_buffer)
                        .unwrap();
            }
            // move down a line
            point += Size::new(0, line_height);
        }
    }

    /// Flush the software buffer onto the physical display
    pub async fn flush(&mut self, delay: &mut impl DelayNs) {
        self.raw_display
            .update(delay, self.frame_buffer.data())
            .await;
    }

    /// Force clear the physical display
    pub async fn clear(&mut self, delay: &mut impl DelayNs) {
        self.raw_display.clear(delay).await;
    }
}
