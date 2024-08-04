use super::{mapper::MapResult, Mapper, Mirroring};

#[derive(Default)]
pub struct DummyMapper;

impl Mapper for DummyMapper {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult> {
        Some(MapResult::Rom { bank: 0, addr })
    }
    fn cpu_map_write(&mut self, addr: u16, _data: u8) -> Option<MapResult> {
        Some(MapResult::Rom { bank: 0, addr })
    }
    fn ppu_map_read(&self, _addr: u16) -> Option<MapResult> {
        None
    }
    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }
}
