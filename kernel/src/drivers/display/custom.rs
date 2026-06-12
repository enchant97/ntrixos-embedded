use embedded_hal_async::{delay::DelayNs, spi::SpiDevice};
// HACK use smaller portion of display until move onto pico2
//pub const WIDTH: usize = 640;
//pub const HEIGHT: usize = 480;
pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 256;
const ROW_SIZE: usize = WIDTH / 8;
pub const BUFFER_SIZE: usize = ROW_SIZE * HEIGHT;

pub struct CustomDisplay<SPI> {
    spi: SPI,
}

impl<SPI> CustomDisplay<SPI>
where
    SPI: SpiDevice,
{
    pub const fn width() -> usize {
        WIDTH
    }

    pub const fn height() -> usize {
        HEIGHT
    }

    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn init(&mut self, delay: &mut impl DelayNs) {
        self.spi.write(&[3]).await.unwrap();
        self.spi.write(&(WIDTH as u32).to_be_bytes()).await.unwrap();
    }

    async fn write_row(&mut self, y: u32, buf: &[u8], delay: &mut impl DelayNs) {
        self.spi.write(&[1]).await.unwrap();
        self.spi.write(&y.to_be_bytes()).await.unwrap();
        self.spi.write(&[0]).await.unwrap();
        for byte in buf {
            // HACK work out whats going on and why this works
            self.spi.write(&[byte.reverse_bits()]).await.unwrap();
        }
        //self.spi.write(buf).await.unwrap();
    }

    pub async fn clear(&mut self, delay: &mut impl DelayNs) {
        for y in 0..Self::height() as u32 {
            self.write_row(y, &[0; ROW_SIZE], delay).await;
        }
    }

    pub async fn update(&mut self, delay: &mut impl DelayNs, new_buffer: &[u8; BUFFER_SIZE]) {
        for y in 0..Self::height() {
            let row_start = y * ROW_SIZE;
            self.write_row(
                y as u32,
                &new_buffer[row_start..row_start + ROW_SIZE],
                delay,
            )
            .await;
        }
    }
}
