use num_enum::TryFromPrimitive;

pub use ntrix_vdc_sdk::charcell::*;
pub use ntrix_vdc_sdk::com;

#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum DisplayOperation {
    GetStat,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DisplayStat {
    pub width: u32,
    pub height: u32,
}

#[derive(PartialEq, Clone, Copy, Debug, TryFromPrimitive)]
#[repr(usize)]
pub enum DisplayCharOperation {
    GetStat,
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct DisplayCharStat {
    pub rows: u32,
    pub cols: u32,
}
