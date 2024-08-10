use std::io::BufRead;

use super::Mapper;
use crate::{
    cartridge::{CartridgeHeader, Mirroring},
    mem::Mem,
};

pub fn build_nrom_mapper(header: &CartridgeHeader, reader: impl BufRead) -> Box<dyn Mapper> {
    let prg_bank_nb = header.prg_rom_banks;
    match prg_bank_nb {
        1 => Box::new(Nrom128::from_reader(reader)),
        2 => Box::new(Nrom256::from_reader(reader)),
        _ => panic!(
            "Unsupported PRG bank number for NROM mapper: {}",
            prg_bank_nb
        ),
    }
}

#[derive(Default)]
pub struct Nrom128 {
    prg_bank: Mem<0x4000>,
    chr_bank: Mem<0x2000>,
}

impl Nrom128 {
    pub fn from_reader(mut reader: impl BufRead) -> Self {
        let mut prg_rom = Mem::default();
        reader.read_exact(&mut prg_rom.as_mut_slice()).unwrap();
        let mut chr_rom = Mem::default();
        reader.read_exact(&mut chr_rom.as_mut_slice()).unwrap();
        Self {
            prg_bank: prg_rom,
            chr_bank: chr_rom,
        }
    }
}

impl Mapper for Nrom128 {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        if addr >= 0x8000 {
            Some(self.prg_bank.read(addr))
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn ppu_map_read(&self, addr: u16) -> Option<u8> {
        if addr < 0x2000 {
            Some(self.chr_bank.read(addr))
        } else {
            None
        }
    }

    fn ppu_map_write(&mut self, addr: u16, _data: u8) -> bool {
        addr < 0x2000
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn ui(&self, _ui: &mut bevy_egui::egui::Ui) {
        todo!()
    }
}

#[derive(Default)]
pub struct Nrom256 {
    prg_rom: Mem<0x8000>,
    chr_rom: Mem<0x2000>,
}

impl Nrom256 {
    pub fn from_reader(mut reader: impl BufRead) -> Self {
        let mut prg_rom = Mem::default();
        reader.read_exact(&mut prg_rom.as_mut_slice()).unwrap();
        let mut chr_rom = Mem::default();
        reader.read_exact(&mut chr_rom.as_mut_slice()).unwrap();
        Self { prg_rom, chr_rom }
    }
}

impl Mapper for Nrom256 {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        if addr >= 0x8000 {
            Some(self.prg_rom.read(addr))
        } else {
            None
        }
    }

    fn cpu_map_write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn ppu_map_read(&self, addr: u16) -> Option<u8> {
        if addr < 0x2000 {
            Some(self.chr_rom.read(addr))
        } else {
            None
        }
    }

    fn ppu_map_write(&mut self, addr: u16, _data: u8) -> bool {
        addr < 0x2000
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn ui(&self, _ui: &mut bevy_egui::egui::Ui) {
        todo!()
    }
}
