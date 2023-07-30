use crate::{cartridge::Cartridge, cpu::Cpu};

#[derive(Default)]
pub(crate) struct Nes {
    cpu: Cpu,
    system_clock: usize,
}

impl Nes {
    pub fn plug_cartridge(&mut self, cartridge: &Cartridge) {
        self.cpu.plug_cartridge(cartridge);
    }

    pub fn step(&mut self) {
        self.cpu.tick();
    }
}
