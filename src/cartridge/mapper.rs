pub struct MappedAddr {
    pub bank: usize,
    pub addr: u16,
}

pub trait Mapper: Send + Sync {
    fn cpu_map_read(&self, addr: u16) -> Option<MappedAddr>;
    fn cpu_map_write(&self, addr: u16, data: u8) -> Option<MappedAddr>;
    fn ppu_map_read(&self, addr: u16) -> Option<MappedAddr>;
    fn ppu_map_write(&self, addr: u16, data: u8) -> Option<MappedAddr>;
}
