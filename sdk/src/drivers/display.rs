#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
#[cfg(feature = "defmt")]
use defmt::bitflags;
use num_enum::TryFromPrimitive;

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

bitflags! {
    #[derive(Default, bytemuck::Pod, bytemuck::Zeroable)]
#[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    #[repr(transparent)]
    pub struct CharAttributes: u8 {
        const INVERT = 1;
    }
}

impl CharAttributes {
    /// Whether to invert the visuals of the cell.
    pub fn invert(&self) -> bool {
        self.contains(CharAttributes::INVERT)
    }
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[non_exhaustive]
pub struct CharCell {
    pub glyph: u8,
    pub attrs: CharAttributes,
}

impl CharCell {
    /// Convert from a u8 glyph, replacing with '?' on when out of ASCII range
    pub const fn from_u8_lossy(glyph: u8) -> Self {
        Self {
            glyph: if glyph.is_ascii() { glyph } else { b'?' },
            attrs: CharAttributes::empty(),
        }
    }
}

impl TryFrom<u8> for CharCell {
    type Error = ();
    fn try_from(glyph: u8) -> Result<Self, Self::Error> {
        if !glyph.is_ascii() {
            Err(())
        } else {
            Ok(Self {
                glyph,
                attrs: CharAttributes::empty(),
            })
        }
    }
}

impl<'a> From<&'a CharCell> for &'a [u8; 2] {
    fn from(value: &'a CharCell) -> Self {
        bytemuck::cast_ref(value)
    }
}

impl CharCell {
    pub fn as_bytes(&self) -> &[u8; 2] {
        bytemuck::cast_ref(self)
    }
}
