use super::{mapper::MappedAddr, Mapper};

#[derive(Default)]
pub struct DummyMapper;

impl Mapper for DummyMapper {
    fn cpu_map_read(&self, addr: u16) -> Option<MappedAddr> {
        Some(MappedAddr { bank: 0, addr })
    }
    fn cpu_map_write(&self, addr: u16, _data: u8) -> Option<MappedAddr> {
        Some(MappedAddr { bank: 0, addr })
    }
    fn ppu_map_read(&self, _addr: u16) -> Option<MappedAddr> {
        None
    }
    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MappedAddr> {
        None
    }
}
