//! Printable character keyboard layout mappings.
//!
//! - only supports Extended ASCII / CP437
//! - used to convert between usage-id and expected character

use sdk::drivers::keyboard::{KeyKind, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedKey {
    Raw(u8),
    Char(u8),
}

impl From<MappedKey> for u8 {
    fn from(value: MappedKey) -> Self {
        match value {
            MappedKey::Raw(v) => v,
            MappedKey::Char(v) => v,
        }
    }
}

impl From<MappedKey> for KeyKind {
    fn from(value: MappedKey) -> Self {
        match value {
            MappedKey::Raw(_) => Self::Raw,
            MappedKey::Char(_) => Self::Char,
        }
    }
}

/// Chosen keyboard layout
pub enum Layout {
    Uk,
    Us,
}

const USAGE_PAGE_LENGTH: usize = 232;
type MappingType = [Option<[u8; 2]>; USAGE_PAGE_LENGTH];

static UK_MAPPINGS: MappingType = const {
    let mut map = [None; USAGE_PAGE_LENGTH];

    // Letters (0x04-0x1D)
    map[0x04] = Some([b'a', b'A']);
    map[0x05] = Some([b'b', b'B']);
    map[0x06] = Some([b'c', b'C']);
    map[0x07] = Some([b'd', b'D']);
    map[0x08] = Some([b'e', b'E']);
    map[0x09] = Some([b'f', b'F']);
    map[0x0A] = Some([b'g', b'G']);
    map[0x0B] = Some([b'h', b'H']);
    map[0x0C] = Some([b'i', b'I']);
    map[0x0D] = Some([b'j', b'J']);
    map[0x0E] = Some([b'k', b'K']);
    map[0x0F] = Some([b'l', b'L']);
    map[0x10] = Some([b'm', b'M']);
    map[0x11] = Some([b'n', b'N']);
    map[0x12] = Some([b'o', b'O']);
    map[0x13] = Some([b'p', b'P']);
    map[0x14] = Some([b'q', b'Q']);
    map[0x15] = Some([b'r', b'R']);
    map[0x16] = Some([b's', b'S']);
    map[0x17] = Some([b't', b'T']);
    map[0x18] = Some([b'u', b'U']);
    map[0x19] = Some([b'v', b'V']);
    map[0x1A] = Some([b'w', b'W']);
    map[0x1B] = Some([b'x', b'X']);
    map[0x1C] = Some([b'y', b'Y']);
    map[0x1D] = Some([b'z', b'Z']);

    // Numbers (0x1E-0x27)
    map[0x1E] = Some([b'1', b'!']);
    map[0x1F] = Some([b'2', b'"']);
    map[0x20] = Some([b'3', b'$']); // uses ASCII replacement, would be '£'
    map[0x21] = Some([b'4', b'$']);
    map[0x22] = Some([b'5', b'%']);
    map[0x23] = Some([b'6', b'^']);
    map[0x24] = Some([b'7', b'&']);
    map[0x25] = Some([b'8', b'*']);
    map[0x26] = Some([b'9', b'(']);
    map[0x27] = Some([b'0', b')']);

    // Control keys (0x28-0x2C)
    map[0x2C] = Some([b' ', b' ']);

    // Symbols (0x2D-0x38)
    map[0x2D] = Some([b'-', b'_']);
    map[0x2E] = Some([b'=', b'+']);
    map[0x2F] = Some([b'[', b'{']);
    map[0x30] = Some([b']', b'}']);
    map[0x31] = Some([b'\\', b'|']);
    map[0x32] = Some([b'#', b'~']);
    map[0x33] = Some([b';', b':']);
    map[0x34] = Some([b'\'', b'@']);
    map[0x35] = Some([b'`', b'`']); // uses ASCII replacement, would be '¬'
    map[0x36] = Some([b',', b'<']);
    map[0x37] = Some([b'.', b'>']);
    map[0x38] = Some([b'/', b'?']);

    map
};

static US_MAPPINGS: MappingType = const {
    let mut map = [None; USAGE_PAGE_LENGTH];

    // Letters (0x04-0x1D)
    map[0x04] = Some([b'a', b'A']);
    map[0x05] = Some([b'b', b'B']);
    map[0x06] = Some([b'c', b'C']);
    map[0x07] = Some([b'd', b'D']);
    map[0x08] = Some([b'e', b'E']);
    map[0x09] = Some([b'f', b'F']);
    map[0x0A] = Some([b'g', b'G']);
    map[0x0B] = Some([b'h', b'H']);
    map[0x0C] = Some([b'i', b'I']);
    map[0x0D] = Some([b'j', b'J']);
    map[0x0E] = Some([b'k', b'K']);
    map[0x0F] = Some([b'l', b'L']);
    map[0x10] = Some([b'm', b'M']);
    map[0x11] = Some([b'n', b'N']);
    map[0x12] = Some([b'o', b'O']);
    map[0x13] = Some([b'p', b'P']);
    map[0x14] = Some([b'q', b'Q']);
    map[0x15] = Some([b'r', b'R']);
    map[0x16] = Some([b's', b'S']);
    map[0x17] = Some([b't', b'T']);
    map[0x18] = Some([b'u', b'U']);
    map[0x19] = Some([b'v', b'V']);
    map[0x1A] = Some([b'w', b'W']);
    map[0x1B] = Some([b'x', b'X']);
    map[0x1C] = Some([b'y', b'Y']);
    map[0x1D] = Some([b'z', b'Z']);

    // Numbers (0x1E-0x27)
    map[0x1E] = Some([b'1', b'!']);
    map[0x1F] = Some([b'2', b'@']);
    map[0x20] = Some([b'3', b'#']);
    map[0x21] = Some([b'4', b'$']);
    map[0x22] = Some([b'5', b'%']);
    map[0x23] = Some([b'6', b'^']);
    map[0x24] = Some([b'7', b'&']);
    map[0x25] = Some([b'8', b'*']);
    map[0x26] = Some([b'9', b'(']);
    map[0x27] = Some([b'0', b')']);

    // Control keys (0x28-0x2C)
    map[0x2C] = Some([b' ', b' ']);

    // Symbols (0x2D-0x38)
    map[0x2D] = Some([b'-', b'_']);
    map[0x2E] = Some([b'=', b'+']);
    map[0x2F] = Some([b'[', b'{']);
    map[0x30] = Some([b']', b'}']);
    map[0x31] = Some([b'\\', b'|']);
    map[0x33] = Some([b';', b':']);
    map[0x34] = Some([b'\'', b'"']);
    map[0x35] = Some([b'`', b'~']);
    map[0x36] = Some([b',', b'<']);
    map[0x37] = Some([b'.', b'>']);
    map[0x38] = Some([b'/', b'?']);

    map
};

pub fn usage_id_to_mapped_key(layout: Layout, usage_id: u8, modifers: Modifiers) -> MappedKey {
    let mapping_table = match layout {
        Layout::Uk => &UK_MAPPINGS,
        Layout::Us => &US_MAPPINGS,
    };
    match mapping_table[usage_id as usize] {
        Some(mapping) => MappedKey::Char(
            mapping[match modifers.shift() {
                false => 0,
                true => 1,
            }],
        ),
        _ => MappedKey::Raw(usage_id),
    }
}
