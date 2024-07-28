use super::mapper::{MappedAddr, Mapper};

#[derive(Default)]
pub struct Nrom128;

impl Mapper for Nrom128 {
    fn cpu_map_read(&self, addr: u16) -> Option<MappedAddr> {
        if addr >= 0x8000 {
            Some(MappedAddr { bank: 0, addr })
        } else {
            None
        }
    }

    fn cpu_map_write(&self, _addr: u16, _data: u8) -> Option<MappedAddr> {
        None
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MappedAddr> {
        if addr < 0x2000 {
            Some(MappedAddr { bank: 0, addr })
        } else {
            None
        }
    }

    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MappedAddr> {
        None
    }
}

#[derive(Default)]
pub struct Nrom256;

impl Mapper for Nrom256 {
    fn cpu_map_read(&self, addr: u16) -> Option<MappedAddr> {
        if addr >= 0x8000 {
            Some(MappedAddr {
                bank: (addr & 0x4000) as usize,
                addr,
            })
        } else {
            None
        }
    }

    fn cpu_map_write(&self, _addr: u16, _data: u8) -> Option<MappedAddr> {
        None
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MappedAddr> {
        if addr < 0x2000 {
            Some(MappedAddr { bank: 0, addr })
        } else {
            None
        }
    }

    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MappedAddr> {
        None
    }
}
