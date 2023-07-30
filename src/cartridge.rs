use std::{cell::RefCell, rc::Rc};

use self::mapper::Mapper;

pub mod mapper;

pub struct Cartridge {
    mapper: Rc<RefCell<dyn Mapper>>,
}

impl Cartridge {
    pub fn new(mapper: impl Mapper + 'static) -> Self {
        Self {
            mapper: Rc::new(RefCell::new(mapper)),
        }
    }

    pub fn from_header(header: &CartridgeHeader) -> Self {
        // TODO : handle error
        let mapper = mapper::create_mapper_by_header(header).expect("failed to create mapper");
        Self { mapper }
    }

    pub fn get_cpu_connector(&self) -> CartridgeConnector {
        CartridgeConnector::Cpu(self.mapper.clone())
    }

    pub fn get_ppu_connector(&self) -> CartridgeConnector {
        CartridgeConnector::Ppu(self.mapper.clone())
    }
}

pub enum CartridgeConnector {
    Cpu(Rc<RefCell<dyn Mapper>>),
    Ppu(Rc<RefCell<dyn Mapper>>),
}

impl CartridgeConnector {
    pub fn read(&self, addr: u16) -> Option<u8> {
        match self {
            CartridgeConnector::Cpu(mapper) => mapper.borrow().prg_read(addr),
            CartridgeConnector::Ppu(mapper) => mapper.borrow().chr_read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match self {
            CartridgeConnector::Cpu(mapper) => mapper.borrow_mut().prg_write(addr, data),
            CartridgeConnector::Ppu(mapper) => mapper.borrow_mut().prg_write(addr, data),
        }
    }
}

pub enum HeaderError {
    ParseError,
}

enum Mirroring {
    Horizontal,
    Vertical,
}

enum ConsoleType {
    Nes,
    VsSystem,
    Playchoice,
    Extended,
}

enum TvSystem {
    PAL,
    NTSC,
    Dual,
}

pub struct INESHeader {
    prg_rom_size: u8,
    prg_ram_size: u8,
    chr_rom_size: u8,
    mapper_id: u8,
    four_screen: bool,
    trainer: bool,
    battery: bool,
    console_type: ConsoleType,
    mirroring: Mirroring,
}

pub struct NES20Header {}

/// NES 2.0 header format
/// could be compatible with iNES ? maybe ?
pub enum CartridgeHeader {
    INES(INESHeader),
    NES20(NES20Header),
}

impl CartridgeHeader {
    pub fn from_bytes(bytes: &[u8; 16]) -> Result<CartridgeHeader, HeaderError> {
        match bytes[7] & 0x0C {
            0x08 => Self::parse_nes2(bytes),
            _ => Self::parse_ines(bytes),
        }
    }

    fn parse_ines(flags: &[u8; 16]) -> Result<CartridgeHeader, HeaderError> {
        Ok(CartridgeHeader::INES(INESHeader {
            prg_rom_size: flags[4],
            prg_ram_size: flags[5],
            chr_rom_size: flags[8],
            mapper_id: flags[6] & 0xF0 >> 4 | flags[7] & 0xF0,
            four_screen: flags[6] & 0x08 != 0,
            trainer: flags[6] & 0x04 != 0,
            battery: flags[6] & 0x02 != 0,
            mirroring: if flags[6] & 0x01 != 0 {
                Mirroring::Vertical
            } else {
                Mirroring::Horizontal
            },
            console_type: match flags[7] & 0x03 {
                0x01 => ConsoleType::VsSystem,
                0x02 => ConsoleType::Playchoice,
                _ => ConsoleType::Nes,
            },
        }))
    }

    fn parse_nes2(bytes: &[u8; 16]) -> Result<CartridgeHeader, HeaderError> {
        unimplemented!("nes2.0 header parsing not implemented");
    }

    pub fn mapper_id(&self) -> u8 {
        match self {
            CartridgeHeader::INES(header) => header.mapper_id,
            CartridgeHeader::NES20(header) => todo!(),
        }
    }

    pub fn prg_rom_size(&self) -> u8 {
        match self {
            CartridgeHeader::INES(header) => header.prg_rom_size,
            CartridgeHeader::NES20(header) => todo!(),
        }
    }
}
