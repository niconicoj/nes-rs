use super::Mapper;
use crate::{cartridge::Mirroring, mem::Mem};

#[derive(Default)]
pub struct DummyMapper {
    bank: Mem<0x8000>,
}

impl Mapper for DummyMapper {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        Some(self.bank.read(addr))
    }
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> bool {
        self.bank.write(addr, data);
        true
    }
    fn ppu_map_read(&self, _addr: u16) -> Option<u8> {
        None
    }
    fn ppu_map_write(&self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn ui(&self, _ui: &mut bevy_egui::egui::Ui) {
        todo!()
    }
}
