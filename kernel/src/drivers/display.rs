use bytemuck::cast_slice;
use embedded_hal_async::{digital::Wait, spi::SpiDevice};
use sdk::drivers::display::{
    CharCell, DisplayCharStat, DisplayStat,
    com::{ControlPacket, DisplayMode, DisplayModeResolution},
};

pub const DISPLAY_STAT: DisplayStat = DisplayStat {
    width: 160,
    height: 120,
};
pub const DISPLAY_CHAR_STAT: DisplayCharStat = DisplayCharStat { rows: 15, cols: 20 };

pub struct DisplayDriver<SPI, BUSY> {
    spi: SPI,
    busy: BUSY,
    mode: DisplayMode,
    pixel_stat: DisplayStat,
    char_stat: Option<DisplayCharStat>,
}

impl<SPI, BUSY> DisplayDriver<SPI, BUSY>
where
    SPI: SpiDevice,
    BUSY: Wait,
{
    pub async fn init(spi: SPI, busy: BUSY) -> Self {
        let mut d = Self {
            spi,
            busy,
            mode: DisplayMode::default(),
            pixel_stat: DisplayStat {
                width: 640,
                height: 320,
            },
            char_stat: None,
        };
        d.reset().await;
        d
    }

    fn update_stats(&mut self) {
        let res = self.mode.resolution();
        self.pixel_stat = DisplayStat {
            width: res.width() as u32,
            height: res.height() as u32,
        };
        self.char_stat = if self.mode.chars_enabled() {
            const FONT_WIDTH: usize = 8;
            const FONT_HEIGHT: usize = 8;
            Some(DisplayCharStat {
                rows: self.pixel_stat.height / FONT_HEIGHT as u32,
                cols: self.pixel_stat.width / FONT_WIDTH as u32,
            })
        } else {
            None
        };
    }

    async fn reset(&mut self) {
        // HACK replace this with actual reset, when user selectable modes is implemented
        self.mode = DisplayMode::new(DisplayModeResolution::R160x120, true);
        let mode_packet = ControlPacket::SetMode(self.mode).pack();
        self.busy.wait_for_low().await.unwrap();
        self.spi.write(&mode_packet).await.unwrap();
        self.update_stats();

        //let reset_packet = ControlPacket::Reset.pack();
        //self.busy.wait_for_low().await.unwrap();
        //self.spi.write(&reset_packet).await.unwrap();
        //self.mode = DisplayMode::default();
    }

    pub async fn get_stat(&self) -> DisplayStat {
        self.pixel_stat
    }

    pub async fn get_char_stat(&self) -> Option<DisplayCharStat> {
        self.char_stat
    }

    pub async fn set_mode(&mut self, new_mode: DisplayMode) {
        let mode_packet = ControlPacket::SetMode(new_mode).pack();
        self.busy.wait_for_low().await.unwrap();
        self.spi.write(&mode_packet).await.unwrap();
        self.update_stats();
    }

    pub async fn flush_pixel(&mut self) {
        todo!()
    }

    /// Flush the char-cell software buffer onto display.
    ///
    /// - Only has effect when character mode is enabled.
    pub async fn flush_char(&mut self, buff: &[CharCell]) {
        if let Some(stat) = self.char_stat {
            let rows = stat.rows as usize;
            let cols = stat.cols as usize;

            if buff.len() != rows * cols {
                panic!("provided buffer not expected size");
            }

            for row_i in 0..rows {
                let row_start = row_i * cols;
                let control_packet = ControlPacket::WriteRowChars(row_i as u16).pack();
                self.busy.wait_for_low().await.unwrap();
                self.spi.write(&control_packet).await.unwrap();
                self.busy.wait_for_low().await.unwrap();
                self.spi
                    .write(cast_slice(&buff[row_start..row_start + cols]))
                    .await
                    .unwrap();
            }
        }
    }
}
