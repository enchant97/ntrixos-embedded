//! ST7920 SPI Driver
//!
//! Based on: <https://github.com/wjakobczyk/st7920/tree/master>
//! Based on: <https://github.com/enchant97/micropython-st7920>

use embedded_hal::digital::OutputPin;
use embedded_hal_async::{delay::DelayNs, spi::SpiBus};

#[repr(u8)]
enum Instruction {
    BasicFunction = 0x30,
    ExtendedFunction = 0x34,
    ClearScreen = 0x01,
    EntryMode = 0x06,
    DisplayOnCursorOff = 0x0C,
    GraphicsOn = 0x36,
    SetGraphicsAddress = 0x80,
}

const INIT_INSTRUCTIONS: [Instruction; 7] = [
    Instruction::BasicFunction,
    Instruction::BasicFunction,
    Instruction::DisplayOnCursorOff,
    Instruction::ClearScreen,
    Instruction::EntryMode,
    Instruction::ExtendedFunction,
    Instruction::GraphicsOn,
];
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;
const ROW_SIZE: usize = WIDTH / 8;
pub const BUFFER_SIZE: usize = ROW_SIZE * HEIGHT;

pub struct ST7920<SPI, CS> {
    spi: SPI,
    cs: CS,
    buffer: [u8; BUFFER_SIZE],
    flip: bool,
}

impl<SPI, CS> ST7920<SPI, CS>
where
    SPI: SpiBus,
    CS: OutputPin,
{
    pub const fn width() -> usize {
        WIDTH
    }

    pub const fn height() -> usize {
        HEIGHT
    }

    pub fn new(spi: SPI, cs: CS, flip: bool) -> Self {
        let buffer = [0; BUFFER_SIZE];
        Self {
            spi,
            cs,
            buffer,
            flip,
        }
    }

    async fn write_command(&mut self, byte: u8, delay: &mut impl DelayNs) {
        self.spi.write(&[0xf8]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[byte & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[(byte << 4) & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
    }

    async fn write_data(&mut self, byte: u8, delay: &mut impl DelayNs) {
        self.spi.write(&[0xf8 | 0x02]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[byte & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
        self.spi.write(&[(byte << 4) & 0xf0]).await.unwrap();
        delay.delay_us(50).await;
    }

    async fn set_graphics_address(&mut self, x: u8, y: u8, delay: &mut impl DelayNs) {
        self.write_command(Instruction::SetGraphicsAddress as u8 | y, delay)
            .await;
        self.write_command(Instruction::SetGraphicsAddress as u8 | x, delay)
            .await;
    }

    pub async fn init(&mut self, delay: &mut impl DelayNs) {
        self.cs.set_high().unwrap();
        for instruction in INIT_INSTRUCTIONS {
            self.write_command(instruction as u8, delay).await;
            delay.delay_ms(2).await;
        }
        self.cs.set_low().unwrap();
    }

    pub async fn clear(&mut self, delay: &mut impl DelayNs) {
        self.cs.set_high().unwrap();
        for y in 0..Self::height() {
            if y < 32 {
                self.set_graphics_address(0, y as u8, delay).await;
            } else {
                self.set_graphics_address(8, y as u8 - 32, delay).await;
            }
            for _ in 0..16 {
                self.write_data(0, delay).await;
            }
        }
        self.buffer.fill(0);
        self.cs.set_low().unwrap();
    }

    #[inline]
    pub async fn update(&mut self, delay: &mut impl DelayNs, new_buffer: &[u8; BUFFER_SIZE]) {
        self.cs.set_high().unwrap();
        for y in 0..Self::height() {
            // check if row needs update
            let row_start = y * ROW_SIZE;
            if self.buffer[row_start..row_start + ROW_SIZE]
                == new_buffer[row_start..row_start + ROW_SIZE]
            {
                continue;
            }
            let row_offset = y * 16;
            if y < 32 {
                self.set_graphics_address(0, y as u8, delay).await;
            } else {
                self.set_graphics_address(8, y as u8 - 32, delay).await;
            }
            for i in 0..16 {
                self.write_data(new_buffer[row_offset + i], delay).await;
            }
        }
        self.buffer.copy_from_slice(new_buffer);
        self.cs.set_low().unwrap();
    }
}
