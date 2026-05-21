#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
#[cfg(feature = "defmt")]
use defmt::bitflags;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum KeyKind {
    Char = 0,
    Raw = 1,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Action {
    Press = 0,
    Release = 1,
}

bitflags! {
    #[derive(Default)]
#[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    #[repr(transparent)]
    pub struct Modifiers: u8 {
        const CTRL = 1;
        const SHIFT = 2;
        const ALT = 4;
        const META= 8;
    }
}

#[allow(unused)]
impl Modifiers {
    pub fn shift(self) -> bool {
        self.contains(Modifiers::SHIFT)
    }
    pub fn ctrl(self) -> bool {
        self.contains(Modifiers::CTRL)
    }
    pub fn alt(self) -> bool {
        self.contains(Modifiers::ALT)
    }
    pub fn meta(self) -> bool {
        self.contains(Modifiers::META)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(C)]
pub struct KeyEvent {
    pub kind: KeyKind,
    /// Either the raw HID code or decoded char (if `KeyKind::Char`)
    pub code: u8,
    pub action: Action,
    pub modifiers: Modifiers,
}
