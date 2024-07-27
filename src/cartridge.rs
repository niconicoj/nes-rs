use std::{fs::File, io::Read};

use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_egui::{egui, EguiContexts};
use mapper::Mapper;

mod dummy;
mod mapper;
mod nrom;

#[cfg(test)]
pub use dummy::DummyMapper;
pub use nrom::Nrom128;
use thiserror::Error;

use crate::nes::NesMarker;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum CartridgeHeader {
    INES(INESHeader),
    NES20(NES20Header),
    Mock,
}

#[derive(Debug, PartialEq)]
pub struct NES20Header {}

#[derive(Debug, PartialEq)]
pub struct INESHeader {
    prg_rom_banks: u8,
    prg_ram_banks: u8,
    chr_rom_banks: u8,
    mapper_id: u8,
    four_screen: bool,
    trainer: bool,
    battery: bool,
    console_type: ConsoleType,
    mirroring: Mirroring,
}

#[derive(Debug, PartialEq)]
enum Mirroring {
    Horizontal,
    Vertical,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum ConsoleType {
    Nes,
    VsSystem,
    Playchoice,
    Extended,
}

#[derive(Debug, Error)]
pub enum HeaderError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
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
            prg_rom_banks: flags[4],
            chr_rom_banks: flags[5],
            prg_ram_banks: flags[8],
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

    fn parse_nes2(_bytes: &[u8; 16]) -> Result<CartridgeHeader, HeaderError> {
        unimplemented!("nes2.0 header parsing not implemented");
    }
}

#[derive(Component)]
pub struct Cartridge {
    header: CartridgeHeader,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    #[cfg(test)]
    pub fn testing() -> Self {
        let mapper: Box<dyn Mapper> = Box::new(DummyMapper::default());
        let header = CartridgeHeader::Mock;

        Self { header, mapper }
    }

    pub fn cpu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.cpu_read(addr)
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> Option<()> {
        self.mapper.cpu_write(addr, data)
    }

    pub fn ppu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.ppu_read(addr)
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> Option<()> {
        self.mapper.ppu_write(addr, data)
    }

    pub fn from_file(file: &str) -> Result<Self, HeaderError> {
        let f = std::fs::File::open(file)?;
        let mut reader = std::io::BufReader::new(f);

        let mut buffer = [0; 16];
        reader.read_exact(&mut buffer)?;
        let header = CartridgeHeader::from_bytes(&buffer)?;

        let mapper = match &header {
            CartridgeHeader::INES(header) => Self::init_ines_mapper(header, reader),
            CartridgeHeader::NES20(_) => todo!("NES 2.0 not implemented"),
            _ => panic!("Invalid header"),
        }?;

        Ok(Self { header, mapper })
    }

    fn init_ines_mapper(
        header: &INESHeader,
        mut reader: std::io::BufReader<File>,
    ) -> Result<Box<dyn Mapper>, HeaderError> {
        if header.trainer {
            let mut trainer = [0; 512];
            reader.read_exact(&mut trainer)?;
        }

        let mut prg_rom = vec![0; header.prg_rom_banks as usize * 0x4000];
        reader.read_exact(&mut prg_rom)?;

        let mut chr_rom = vec![0; header.chr_rom_banks as usize * 0x2000];
        reader.read_exact(&mut chr_rom)?;

        let mapper = match header.mapper_id {
            0x00 if header.prg_rom_banks == 1 => {
                Box::new(Nrom128::new(prg_rom, chr_rom)) as Box<dyn Mapper>
            }
            _ => todo!(),
        };

        Ok(mapper)
    }
}

pub struct CartridgePlugin;

impl Plugin for CartridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            cartridge_ui.run_if(input_toggle_active(false, KeyCode::KeyI)),
        );
    }
}

pub fn cartridge_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    cartridge: Query<(Entity, Option<&Cartridge>, &NesMarker)>,
) {
    egui::Window::new("Cartridge").show(contexts.ctx_mut(), |ui| match cartridge.get_single() {
        Ok((_, Some(cartridge), _)) => match &cartridge.header {
            CartridgeHeader::INES(header) => {
                ui.label(format!("mapper {}", header.mapper_id));
                cartridge.mapper.ui(ui);
            }
            _ => todo!(),
        },
        Ok((entity, None, _)) => {
            ui.label("No cartridge inserted");
            if ui.button("Load test cartridge").clicked() {
                let cartridge =
                    Cartridge::from_file("assets/nestest.nes").expect("Failed to load cartridge");
                commands.entity(entity).insert(cartridge);
            }
        }
        _ => {}
    });
}
