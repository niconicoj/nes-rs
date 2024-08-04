use std::{fs::File, io::Read};

use bevy::prelude::*;
use bevy_egui::egui::{ScrollArea, Separator};
use bevy_egui::{egui, EguiContexts};
use mapper::{MapResult, Mapper};

mod dummy;
mod mapper;
mod mmc1;
mod nrom;

#[cfg(test)]
pub use dummy::DummyMapper;
use mmc1::Mmc1;
pub use nrom::Nrom128;
use nrom::Nrom256;
use thiserror::Error;

use crate::mem::Mem;
use crate::nes::NesMarker;

#[derive(Default, Debug, PartialEq)]
pub struct CartridgeHeader {
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

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
    OneScreenLo,
    OneScreenHi,
}

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq)]
enum ConsoleType {
    #[default]
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
    #[cfg(test)]
    pub fn with_mirroring(mirroring: Mirroring) -> Self {
        let mut h = Self::default();
        h.mirroring = mirroring;
        h
    }

    pub fn from_bytes(bytes: &[u8; 16]) -> Result<Self, HeaderError> {
        match bytes[7] & 0x0C {
            0x08 => Self::parse_nes2(bytes),
            _ => Self::parse_ines(bytes),
        }
    }

    fn parse_ines(flags: &[u8; 16]) -> Result<Self, HeaderError> {
        Ok(CartridgeHeader {
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
        })
    }

    fn parse_nes2(_bytes: &[u8; 16]) -> Result<CartridgeHeader, HeaderError> {
        unimplemented!("nes2.0 header parsing not implemented");
    }
}

#[derive(Component)]
pub struct Cartridge {
    header: CartridgeHeader,
    prg_banks: Vec<Mem<0x4000>>,
    chr_banks: Vec<Mem<0x2000>>,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    #[cfg(test)]
    pub fn testing(header: Option<CartridgeHeader>) -> Self {
        let mapper: Box<dyn Mapper> = Box::new(DummyMapper::default());
        let header = header.unwrap_or(CartridgeHeader::default());

        Self {
            header,
            mapper,
            prg_banks: vec![Mem::default(); 2],
            chr_banks: vec![Mem::default(); 1],
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring().unwrap_or(self.header.mirroring)
    }

    pub fn cpu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.cpu_map_read(addr).map(|result| match result {
            MapResult::Rom { bank, addr } => self.prg_banks[bank].read(addr),
            MapResult::Instant { data } => data,
        })
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        if let Some(MapResult::Rom { bank, addr }) = self.mapper.cpu_map_write(addr, data) {
            self.prg_banks[bank].write(addr, data);
            true
        } else {
            false
        }
    }

    pub fn ppu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.ppu_map_read(addr).map(|result| match result {
            MapResult::Rom { bank, addr } => self.chr_banks[bank].read(addr),
            MapResult::Instant { data } => data,
        })
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        if let Some(MapResult::Rom { bank, addr }) = self.mapper.ppu_map_write(addr, data) {
            self.chr_banks[bank].write(addr, data);
            true
        } else {
            false
        }
    }

    pub fn from_file(file: &str) -> Result<Self, HeaderError> {
        let f = std::fs::File::open(file)?;
        let mut reader = std::io::BufReader::new(f);

        let mut buffer = [0; 16];
        reader.read_exact(&mut buffer)?;
        let header = CartridgeHeader::from_bytes(&buffer)?;
        let mapper = Self::init_mapper(&header, &mut reader)?;

        let mut prg_banks = vec![Mem::default(); header.prg_rom_banks as usize];
        for bank in &mut prg_banks {
            reader.read_exact(bank.as_mut_slice())?;
        }

        let mut chr_banks = vec![Mem::default(); header.chr_rom_banks as usize];
        for bank in &mut chr_banks {
            reader.read_exact(bank.as_mut_slice())?;
        }

        Ok(Self {
            chr_banks,
            prg_banks,
            mapper,
            header,
        })
    }

    fn init_mapper(
        cartridge: &CartridgeHeader,
        reader: &mut std::io::BufReader<File>,
    ) -> Result<Box<dyn Mapper>, HeaderError> {
        if cartridge.trainer {
            let mut trainer = [0; 512];
            reader.read_exact(&mut trainer)?;
        }

        let mapper = match cartridge.mapper_id {
            0x00 if cartridge.prg_rom_banks == 1 => Box::new(Nrom128::default()) as Box<dyn Mapper>,
            0x00 if cartridge.prg_rom_banks == 2 => Box::new(Nrom256::default()) as Box<dyn Mapper>,
            0x01 => {
                let mmc1 = Mmc1::new(
                    cartridge.prg_rom_banks as usize,
                    cartridge.chr_rom_banks as usize,
                );
                Box::new(mmc1) as Box<dyn Mapper>
            }
            _ => todo!("mapper {} is not implemented yet", cartridge.mapper_id),
        };

        Ok(mapper)
    }
}

pub fn cartridge_gui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    query: Query<(Entity, Option<&Cartridge>, &NesMarker)>,
) {
    if let Ok((entity, maybe_cartridge, _)) = query.get_single() {
        egui::Window::new("Cartridge")
            .min_width(420.0)
            .show(contexts.ctx_mut(), |ui| match maybe_cartridge {
                Some(cartridge) => {
                    ui.heading(format!("mapper {}", cartridge.header.mapper_id));
                    ui.separator();
                    ui.label("PRG banks");
                    ui.monospace("         0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");
                    ui.add(Separator::default().spacing(2.0));
                    let text_style = egui::TextStyle::Monospace;
                    let row_height = ui.text_style_height(&text_style);
                    let total_rows = 0x8000 / 16;
                    ui.push_id("prg_memory", |ui| {
                        ScrollArea::vertical()
                            .auto_shrink(false)
                            .max_height(200.0)
                            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                                for row in row_range {
                                    let start = (0x8000 + row * 16) as u16;
                                    let end = start + 16;
                                    let row_text = (start..end)
                                        .map(|addr| {
                                            cartridge
                                                .cpu_read(addr)
                                                .map_or("XX".to_string(), |v| format!("{:02X}", v))
                                        })
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    ui.monospace(format!("${:#06X}: {}", start, row_text));
                                }
                            });
                    });
                }
                None => {
                    ui.label("No cartridge inserted");
                    if ui.button("Load test cartridge").clicked() {
                        let cartridge = Cartridge::from_file("assets/nestest.nes")
                            .expect("Failed to load cartridge");
                        commands.entity(entity).insert(cartridge);
                    }
                }
            });
    }
}
