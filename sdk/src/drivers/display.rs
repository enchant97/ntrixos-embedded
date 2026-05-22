#[repr(usize)]
#[derive(PartialEq, Clone, Copy, Debug)]
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
