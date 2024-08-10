use std::io::Read;

use bevy::prelude::*;
use bevy_egui::egui::{ScrollArea, Separator};
use bevy_egui::{egui, EguiContexts};
use mapper::{build_mapper, Mapper};

mod mapper;

use thiserror::Error;

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
    Horizontal = 0x03,
    Vertical = 0x02,
    OneScreenLo = 0x00,
    OneScreenHi = 0x01,
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
        let raw_display = bytes
            .iter()
            .map(|b| format!("{:#04x} ({:#010b})", b, b))
            .collect::<Vec<_>>()
            .join("\n");
        debug!("Parsing header \n{}", raw_display);
        match bytes[7] & 0x0C {
            0x0C => Self::parse_nes2(bytes),
            _ => Self::parse_ines(bytes),
        }
    }

    fn parse_ines(flags: &[u8; 16]) -> Result<Self, HeaderError> {
        debug!("Parsing iNES header");
        Ok(CartridgeHeader {
            prg_rom_banks: flags[4],
            chr_rom_banks: flags[5],
            prg_ram_banks: flags[8],
            mapper_id: ((flags[6] & 0xF0) >> 4) | (flags[7] & 0xF0),
            four_screen: flags[6] & 0x08 != 0,
            trainer: flags[6] & 0x04 != 0,
            battery: flags[6] & 0x02 == 0,
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
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    #[cfg(test)]
    pub fn testing(header: Option<CartridgeHeader>) -> Self {
        let mapper = mapper::dummy();
        let header = header.unwrap_or(CartridgeHeader::default());

        Self { header, mapper }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring().unwrap_or(self.header.mirroring)
    }

    pub fn cpu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.cpu_map_read(addr)
    }

    #[must_use]
    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        self.mapper.cpu_map_write(addr, data)
    }

    pub fn ppu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.ppu_map_read(addr)
    }

    #[must_use]
    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        self.mapper.ppu_map_write(addr, data)
    }

    pub fn from_file(file: &str) -> Result<Self, HeaderError> {
        let f = std::fs::File::open(file)?;
        let mut reader = std::io::BufReader::new(f);

        let mut buffer = [0; 16];
        reader.read_exact(&mut buffer)?;
        let header = CartridgeHeader::from_bytes(&buffer)?;

        if header.trainer {
            debug!("Reading trainer");
            let mut trainer = [0; 512];
            reader.read_exact(&mut trainer)?;
        }

        info!("Mapper ID {}", header.mapper_id);
        let mapper = build_mapper(&header, reader)?;
        Ok(Self { mapper, header })
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
                    cartridge.mapper.ui(ui);
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
