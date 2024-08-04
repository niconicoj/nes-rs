use super::Mirroring;

pub enum MapResult {
    Rom { bank: usize, addr: u16 },
    Instant { data: u8 },
}

pub trait Mapper: Send + Sync {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult>;
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<MapResult>;
    fn ppu_map_read(&self, addr: u16) -> Option<MapResult>;
    fn ppu_map_write(&self, addr: u16, data: u8) -> Option<MapResult>;
    fn mirroring(&self) -> Option<Mirroring>;
}
