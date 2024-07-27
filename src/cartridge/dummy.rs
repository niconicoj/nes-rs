use bevy_egui::egui;

use crate::mem::Mem;

use super::Mapper;

#[derive(Default)]
pub struct DummyMapper {
    prg_ram: Mem<0x10000>,
    chr_ram: Mem<0x10000>,
}

impl Mapper for DummyMapper {
    fn cpu_read(&self, addr: u16) -> Option<u8> {
        Some(self.prg_ram.read(addr))
    }
    fn cpu_write(&mut self, addr: u16, data: u8) -> Option<()> {
        self.prg_ram.write(addr, data);
        Some(())
    }
    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.chr_ram.read(addr))
    }
    fn ppu_write(&mut self, addr: u16, data: u8) -> Option<()> {
        self.chr_ram.write(addr, data);
        Some(())
    }

    fn ui(&self, _: &mut egui::Ui) {}
}
