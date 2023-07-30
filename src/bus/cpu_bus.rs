use tracing::info;

use crate::{
    cartridge::{Cartridge, CartridgeConnector},
    ppu::Ppu,
    ram::Ram,
};

#[derive(Default)]
pub struct CpuBus {
    wram: Ram<0x800>,
    ppu: Ppu,
    cartridge: Option<CartridgeConnector>,
}

impl CpuBus {
    pub fn plug_cartridge(&mut self, cartridge: &Cartridge) {
        self.cartridge = Some(cartridge.get_cpu_connector());
    }

    pub fn unplug_cartridge(&mut self) {
        self.cartridge = None;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match (addr, &self.cartridge) {
            (0x0000..=0x1FFF, _) => self.wram.read(addr),
            (0x4020..=0xFFFF, Some(cartridge)) => match cartridge.read(addr) {
                Some(data) => data,
                None => self.open_bus(),
            },
            _ => self.open_bus(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match (addr, &mut self.cartridge) {
            (0x0000..=0x1FFF, _) => self.wram.write(addr, data),
            (0x4020..=0xFFFF, Some(cartridge)) => cartridge.write(addr, data),
            _ => (),
        };
    }

    fn open_bus(&self) -> u8 {
        info!("reading open bus");
        0
    }
}
