use num_enum::TryFromPrimitive;

pub use ntrix_vdc_sdk::charcell::*;
pub use ntrix_vdc_sdk::com;

#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum DisplayOperation {
    GetMode,
    SetModePixel,
    SetModeCharacter,
    GetStat,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DisplayMode {
    Pixel,
    Character,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DisplayStat {
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub char_rows: u32,
    pub char_cols: u32,
}
