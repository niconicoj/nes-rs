use bitfield::bitfield;
use std::{array, fmt::Display};

bitfield! {
    #[derive(Copy, Clone, Default, Eq,PartialEq)]
    pub struct OamEntry(u32);
    impl Debug;
    pub y, set_y: 7, 0;
    pub tile_id, set_tile_id: 15, 8;
    pub attribute, set_attribute: 23, 16;
    pub x, set_x: 31, 24;
}

impl Display for OamEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{})\tID: {:#04x}, AT: {:#04x}",
            self.x(),
            self.y(),
            self.tile_id(),
            self.attribute()
        )
    }
}

pub struct Oam {
    entries: [OamEntry; 64],
}

pub struct OamIterator<'a> {
    oam: &'a Oam,
    index: usize,
}

impl<'a> Iterator for OamIterator<'a> {
    type Item = &'a OamEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.oam.entries.get(self.index - 1)
    }
}

impl Default for Oam {
    fn default() -> Self {
        Self {
            entries: array::from_fn(|_| OamEntry::default()),
        }
    }
}

impl Oam {
    pub fn iter(&self) -> OamIterator {
        OamIterator {
            oam: self,
            index: 0,
        }
    }
    pub fn get_entry(&self, index: u8) -> OamEntry {
        let index = index & 0x3F;
        unsafe { OamEntry(self.entries.get_unchecked(index as usize).0) }
    }

    pub fn write_byte(&mut self, index: u8, data: u8) {
        let entry_index = (index >> 2) as usize;
        let entry = unsafe { self.entries.get_unchecked_mut(entry_index) };
        let entry_offset = (index & 0x3) << 3;
        entry.0 &= !(0xFFu32 << entry_offset);
        entry.0 |= (data as u32) << entry_offset;
    }
}

#[cfg(test)]
mod tests {
    use super::{Oam, OamEntry};

    #[test]
    fn iter() {
        let mut oam = Oam::default();

        oam.entries[0] = OamEntry(0x01234567);
        oam.entries[1] = OamEntry(0x89ABCDEF);

        let mut iter = oam.iter();

        assert_eq!(iter.next(), Some(&OamEntry(0x01234567)));
        assert_eq!(iter.next(), Some(&OamEntry(0x89ABCDEF)));
        assert_eq!(iter.next(), Some(&OamEntry(0x00000000)));
    }

    #[test]
    fn set_byte() {
        let mut oam = Oam::default();

        oam.write_byte(0x00, 0x01);
        assert_eq!(oam.entries[0].y(), 0x01);
        assert_eq!(oam.entries[0].tile_id(), 0x00);
        assert_eq!(oam.entries[0].attribute(), 0x00);
        assert_eq!(oam.entries[0].x(), 0x00);
        assert!(oam.entries.iter().skip(1).all(|entry| entry.0 == 0x00));

        oam.write_byte(0x01, 0x02);
        assert_eq!(oam.entries[0].y(), 0x01);
        assert_eq!(oam.entries[0].tile_id(), 0x02);
        assert_eq!(oam.entries[0].attribute(), 0x00);
        assert_eq!(oam.entries[0].x(), 0x00);

        oam.write_byte(0x02, 0x03);
        assert_eq!(oam.entries[0].y(), 0x01);
        assert_eq!(oam.entries[0].tile_id(), 0x02);
        assert_eq!(oam.entries[0].attribute(), 0x03);
        assert_eq!(oam.entries[0].x(), 0x00);

        oam.write_byte(0x03, 0x7F);
        assert_eq!(oam.entries[0].y(), 0x01);
        assert_eq!(oam.entries[0].tile_id(), 0x02);
        assert_eq!(oam.entries[0].attribute(), 0x03);
        assert_eq!(oam.entries[0].x(), 0x7F);

        oam.write_byte(0x03, 0x7F);
        assert_eq!(oam.entries[0].x(), 0x7F);
        oam.write_byte(0x00, 0x7F);
        assert_eq!(oam.entries[0].y(), 0x7F);

        oam.write_byte(0x04, 0xFF);
        assert_eq!(oam.entries[0].y(), 0x7F);
        assert_eq!(oam.entries[0].tile_id(), 0x02);
        assert_eq!(oam.entries[0].attribute(), 0x03);
        assert_eq!(oam.entries[0].x(), 0x7F);
        assert_eq!(oam.entries[1].y(), 0xFF);
    }
}
