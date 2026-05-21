use defmt::bitflags;

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Key {
    /// printable character
    Char(u8),
    /// raw usage id
    Raw(u8),
}

bitflags! {
    #[derive(Default)]
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

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct KeyEvent {
    pub key: Key,
    pub action: Action,
    pub modifiers: Modifiers,
}
