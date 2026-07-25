use bytemuck::cast_slice;
use embedded_hal_async::{delay::DelayNs, digital::Wait, spi::SpiDevice};
use sdk::drivers::display::{
    CharCell, DisplayStat,
    com::{ControlPacket, DisplayMode, DisplayModeResolution},
};
use static_cell::StaticCell;

const PIXEL_HEIGHT: usize = 120;
const PIXEL_WIDTH: usize = 160;
const PIXEL_BUFFER_SIZE: usize = (PIXEL_WIDTH / 8) * PIXEL_HEIGHT;
const CHAR_ROWS: usize = 15;
const CHAR_COLS: usize = 20;
const CHAR_BUFFER_SIZE: usize = CHAR_ROWS * CHAR_COLS;
pub const DISPLAY_STAT: DisplayStat = DisplayStat {
    pixel_width: PIXEL_WIDTH as u32,
    pixel_height: PIXEL_HEIGHT as u32,
    char_rows: CHAR_ROWS as u32,
    char_cols: CHAR_COLS as u32,
};

static PIXEL_BUFFER: StaticCell<[u8; PIXEL_BUFFER_SIZE]> = StaticCell::new();
static CHARACTER_BUFFER: StaticCell<[CharCell; CHAR_BUFFER_SIZE]> = StaticCell::new();

pub struct DisplayDriver<SPI, BUSY> {
    spi: SPI,
    busy: BUSY,
    pixel_buffer: &'static mut [u8; PIXEL_BUFFER_SIZE],
    char_buffer: &'static mut [CharCell; CHAR_BUFFER_SIZE],
}

impl<SPI, BUSY> DisplayDriver<SPI, BUSY>
where
    SPI: SpiDevice,
    BUSY: Wait,
{
    pub fn new(spi: SPI, busy: BUSY) -> Self {
        Self {
            spi,
            busy,
            pixel_buffer: PIXEL_BUFFER.init_with(|| [0u8; PIXEL_BUFFER_SIZE]),
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

    /// Setup the display
    pub async fn init(&mut self) {
        let mode_packet =
            ControlPacket::SetMode(DisplayMode::new(DisplayModeResolution::R160x120, true)).pack();
        self.busy.wait_for_low().await.unwrap();
        self.spi.write(&mode_packet).await.unwrap();
    }

    /// Flush the software buffer onto display
    pub async fn flush(&mut self, delay: &mut impl DelayNs) {
        for row_i in 0..CHAR_ROWS {
            let row_start = row_i * CHAR_COLS;
            let control_packet = ControlPacket::WriteRowChars(row_i as u16).pack();
            self.busy.wait_for_low().await.unwrap();
            self.spi.write(&control_packet).await.unwrap();
            self.busy.wait_for_low().await.unwrap();
            self.spi
                .write(cast_slice(
                    &self.char_buffer[row_start..row_start + CHAR_COLS],
                ))
                .await
                .unwrap();
        }
    }
}
