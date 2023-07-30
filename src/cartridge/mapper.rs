use std::{cell::RefCell, rc::Rc};

use mockall::automock;

use super::CartridgeHeader;

pub mod nrom;

#[automock]
pub trait Mapper {
    fn prg_read(&self, addr: u16) -> Option<u8>;
    fn prg_write(&mut self, addr: u16, data: u8);
    fn chr_read(&self, addr: u16) -> Option<u8>;
    fn chr_write(&mut self, addr: u16, data: u8);
}

#[derive(Debug)]
pub enum MapperError {
    Unknown,
    Invalid,
}

pub fn create_mapper_by_header(
    header: &CartridgeHeader,
) -> Result<Rc<RefCell<dyn Mapper>>, MapperError> {
    match header.mapper_id() {
        0x0 => match header.prg_rom_size() {
            1 => Ok(Rc::new(RefCell::new(nrom::NRom128::default()))),
            2 => Ok(Rc::new(RefCell::new(nrom::NRom256::default()))),
            _ => Err(MapperError::Invalid),
        },
        _ => Err(MapperError::Unknown),
    }
}
