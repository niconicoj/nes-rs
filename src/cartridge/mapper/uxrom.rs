use bevy::log::debug;

use super::{MapResult, Mapper};
use crate::cartridge::Mirroring;

pub struct Uxrom {
    prg_bank_nb: usize,
    chr_bank_nb: usize,
    vram: [u8; 0x2000],
    bank_select: usize,
}

impl Uxrom {
    pub fn new(prg_bank_nb: usize, chr_bank_nb: usize) -> Self {
        Self {
            prg_bank_nb,
            chr_bank_nb,
            vram: [0; 0x2000],
            bank_select: 0,
        }
    }
}

impl Mapper for Uxrom {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult> {
        match addr {
            0x6000..=0x7FFF => Some(MapResult::Instant {
                data: self.vram[(addr & 0x1FFF) as usize],
            }),
            0x8000..=0xBFFF => Some(MapResult::Rom {
                bank: self.bank_select,
                addr,
            }),
            0xC000..=0xFFFF => Some(MapResult::Rom {
                bank: self.prg_bank_nb - 1,
                addr,
            }),
            _ => None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<MapResult> {
        match addr {
            0x6000..=0x7FFF => self.vram[(addr & 0x1FFF) as usize] = data,
            0x8000..=0xFFFF => {
                // bus conflict
                self.bank_select = (data & 0x07) as usize;
            }
            _ => {}
        };
        None
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr < 0x2000 {
            Some(MapResult::Rom { bank: 0, addr })
        } else {
            None
        }
    }

    fn ppu_map_write(&self, addr: u16, _data: u8) -> Option<MapResult> {
        if addr < 0x2000 && self.chr_bank_nb == 0 {
            debug!("Write to CHR Uxrom mapper: addr: {:#X}", addr);
            Some(MapResult::Rom { bank: 0, addr })
        } else {
            None
        }
    }

    fn mirroring(&self) -> Option<Mirroring> {
        None
    }

    fn ui(&self, ui: &mut bevy_egui::egui::Ui) {
        ui.monospace(format!("Selected bank : {}", self.bank_select));
    }
}
