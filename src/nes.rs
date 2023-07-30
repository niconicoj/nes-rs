use crate::{cartridge::Cartridge, cpu::Cpu};

#[derive(Default)]
pub(crate) struct Nes {
    cpu: Cpu,
    global_clock: usize,
}

impl Nes {
    pub fn plug_cartridge(&mut self, cartridge: &Cartridge) {
        self.cpu.plug_cartridge(cartridge);
    }

    pub fn unplug_cartridge(&mut self) {
        self.cpu.unplug_cartridge();
    }

    pub fn tick(&mut self) {
        self.cpu.tick();
    }
}
