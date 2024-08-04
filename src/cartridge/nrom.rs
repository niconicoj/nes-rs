use super::{
    mapper::{MapResult, Mapper},
    Mirroring,
};

#[derive(Default)]
pub struct Nrom128;

impl Mapper for Nrom128 {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr >= 0x8000 {
            Some(MapResult::Rom { bank: 0, addr })
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr < 0x2000 {
            Some(MapResult::Rom { bank: 0, addr })
        } else {
            None
        }
    }

    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }
}

#[derive(Default)]
pub struct Nrom256;

impl Mapper for Nrom256 {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr >= 0x8000 {
            Some(MapResult::Rom {
                bank: (addr & 0x4000) as usize,
                addr,
            })
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr < 0x2000 {
            Some(MapResult::Rom { bank: 0, addr })
        } else {
            None
        }
    }

    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }
}
