use std::io::BufRead;

use super::{CartridgeHeader, HeaderError, Mirroring};
use bevy_egui::egui::Ui;

mod dummy;
mod mmc1;
mod nrom;
// mod uxrom;

pub trait Mapper: Send + Sync {
    fn cpu_map_read(&self, addr: u16) -> Option<u8>;
    fn cpu_map_write(&mut self, addr: u16, data: u8) -> bool;
    fn ppu_map_read(&self, addr: u16) -> Option<u8>;
    fn ppu_map_write(&self, addr: u16, data: u8) -> bool;
    fn mirroring(&self) -> Option<Mirroring>;
    fn ui(&self, ui: &mut Ui);
}

#[cfg(test)]
pub fn dummy() -> Box<dyn Mapper> {
    Box::new(dummy::DummyMapper::default())
}

pub fn build_mapper(
    cartridge: &CartridgeHeader,
    reader: impl BufRead,
) -> Result<Box<dyn Mapper>, HeaderError> {
    let mapper = match cartridge.mapper_id {
        0x00 => nrom::build_nrom_mapper(cartridge, reader),
        0x01 => mmc1::build_mmc1_mapper(cartridge, reader),
        // 0x02 => mmc1::build_uxrom_mapper(cartridge, reader),
        _ => todo!("mapper {} is not implemented yet", cartridge.mapper_id),
    };

    Ok(mapper)
}
