use std::io::BufRead;

use bevy::log::info;

use super::Mapper;
use crate::{
    cartridge::{CartridgeHeader, Mirroring},
    mem::Mem,
};

pub fn build_uxrom_mapper(header: &CartridgeHeader, mut reader: impl BufRead) -> Box<dyn Mapper> {
    info!("PRG banks: {}", header.prg_rom_banks);
    let mut prg_banks = vec![Mem::default(); header.prg_rom_banks as usize];
    for bank in prg_banks.iter_mut() {
        reader.read_exact(&mut bank.as_mut_slice()).unwrap();
    }

    let mut chr_bank = Mem::default();
    reader.read_exact(&mut chr_bank.as_mut_slice()).unwrap();

    let prg_ram_bank = if header.prg_ram_banks > 0 {
        Some(Mem::default())
    } else {
        None
    };

    Box::new(Uxrom::new(prg_banks, chr_bank, prg_ram_bank, true))
}

pub struct Uxrom {
    prg_banks: Vec<Mem<0x4000>>,
    chr_bank: Mem<0x2000>,
    vram: Option<Mem<0x2000>>,
    bank_select: usize,
    bus_conflict: bool,
}

impl Uxrom {
    pub fn new(
        prg_banks: Vec<Mem<16384>>,
        chr_bank: Mem<8192>,
        vram: Option<Mem<8192>>,
        bus_conflict: bool,
    ) -> Self {
        Self {
            prg_banks,
            chr_bank,
            vram,
            bank_select: 0,
            bus_conflict: true,
        }
    }
}

impl Mapper for Uxrom {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        match addr {
            0x6000..=0x7FFF => self.vram.as_ref().map(|bank| bank.read(addr & 0x1FFF)),
            0x8000..=0xBFFF => self
                .prg_banks
                .get(self.bank_select)
                .map(|bank| bank.read(addr & 0x3FFF)),
            0xC000..=0xFFFF => self.prg_banks.last().map(|bank| bank.read(addr & 0x3FFF)),
            _ => None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x6000..=0x7FFF => {
                if let Some(prg_ram) = self.vram.as_mut() {
                    prg_ram.write(addr, data);
                    true
                } else {
                    false
                }
            }
            0x8000..=0xFFFF => {
                if self.bus_conflict {
                    let cart_data = self.prg_banks[self.bank_select].read(addr & 0x3FFF);
                    self.bank_select = (data & cart_data & 0x07) as usize;
                } else {
                    self.bank_select = data as usize & 0x07;
                }
                true
            }
            _ => false,
        }
    }

    fn ppu_map_read(&self, addr: u16) -> Option<u8> {
        if addr < 0x2000 {
            Some(self.chr_bank.read(addr))
        } else {
            None
        }
    }

    fn ppu_map_write(&mut self, addr: u16, data: u8) -> bool {
        if addr < 0x2000 {
            self.chr_bank.write(addr, data);
            true
        } else {
            false
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn ui(&self, ui: &mut bevy_egui::egui::Ui) {
        ui.monospace(format!("Selected bank : {}", self.bank_select));
    }
}
