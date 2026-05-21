#[repr(usize)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DisplayOperation {
    GetMode,
    SetMode,
    GetStat,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DisplayMode {
    Text,
    Graphics,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DisplayStat {
    /// depending on current display mode: characters or pixels
    pub width: u32,
    /// depending on current display mode: characters or pixels
    pub height: u32,
}
