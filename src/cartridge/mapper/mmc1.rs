use std::io::BufRead;

use bevy::log::{debug, info};
use bitfield::bitfield;

use super::Mapper;
use crate::{
    cartridge::{CartridgeHeader, Mirroring},
    mem::Mem,
};

pub fn build_mmc1_mapper(header: &CartridgeHeader, mut reader: impl BufRead) -> Box<dyn Mapper> {
    info!("PRG banks: {}", header.prg_rom_banks);
    let mut prg_banks = vec![Mem::default(); header.prg_rom_banks as usize];
    for bank in prg_banks.iter_mut() {
        reader.read_exact(&mut bank.as_mut_slice()).unwrap();
    }

    info!("CHR banks: {}", header.chr_rom_banks);
    let mut chr_banks = vec![Mem::default(); header.chr_rom_banks as usize];
    for bank in chr_banks.iter_mut() {
        reader.read_exact(&mut bank.as_mut_slice()).unwrap();
    }

    let prg_ram_bank = if header.prg_ram_banks > 0 {
        Some(Mem::default())
    } else {
        None
    };

    Box::new(Mmc1::new(
        prg_banks,
        chr_banks,
        prg_ram_bank,
        header.mirroring,
    ))
}

bitfield! {
    struct ControlRegister(u8);
    mirroring, _: 1, 0;
    prg_mode, _: 3, 2;
    chr_mode, _: 4, 4;
}

impl Default for ControlRegister {
    fn default() -> Self {
        Self(0x0C)
    }
}

impl ControlRegister {
    fn reset(&mut self) {
        self.0 = self.0 | 0x0C;
    }
}

pub struct Mmc1 {
    control_register: ControlRegister,
    shift_register: u8,
    shift_count: u8,
    chr_bank_hi: usize,
    chr_bank_lo: usize,
    prg_bank: usize,
    vram: Option<Mem<0x2000>>,
    prg_banks: Vec<Mem<0x4000>>,
    chr_banks: Vec<Mem<0x2000>>,
}

impl Mmc1 {
    pub fn new(
        prg_rom_banks: Vec<Mem<0x4000>>,
        chr_rom_banks: Vec<Mem<0x2000>>,
        prg_ram_bank: Option<Mem<0x2000>>,
        mirroring: Mirroring,
    ) -> Self {
        debug!("MMC1 mapper mirroring: {:?}", mirroring as u8);
        Self {
            control_register: ControlRegister(0x1C | mirroring as u8),
            shift_register: 0,
            shift_count: 0,
            chr_bank_hi: 0,
            chr_bank_lo: 0,
            prg_bank: 0,
            vram: prg_ram_bank,
            prg_banks: prg_rom_banks,
            chr_banks: chr_rom_banks,
        }
    }
}

impl Mmc1 {
    fn load_shift_register(&mut self, data: u8) -> u8 {
        let loaded_value = self.shift_register >> 1 | ((data & 0x01) << 4);
        self.shift_count = 0;
        self.shift_register = 0;
        loaded_value
    }
}

impl Mapper for Mmc1 {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        debug!(
            "MMC1 read at {:#06x}, mode : {:#06x}",
            addr,
            self.control_register.prg_mode()
        );
        match (addr, self.control_register.prg_mode()) {
            (0x6000..=0x7FFF, _) => self.vram.as_ref().map(|bank| bank.read(addr & 0x1FFF)),
            // full bank
            (0x8000..=0xBFFF, 0) | (0x8000..=0xBFFF, 1) => self
                .prg_banks
                .get(self.prg_bank & 0xFE)
                .map(|bank| bank.read(addr)),
            (0xC000..=0xFFFF, 0) | (0xC000..=0xFFFF, 1) => self
                .prg_banks
                .get(self.prg_bank & 0xFE + 1)
                .map(|bank| bank.read(addr)),
            // first bank fixed
            (0x8000..=0xBFFF, 2) => self.prg_banks.first().map(|bank| bank.read(addr)),
            (0xC000..=0xFFFF, 2) => self
                .prg_banks
                .get(self.prg_bank)
                .map(|bank| bank.read(addr)),
            // last bank fixed
            (0x8000..=0xBFFF, 3) => self
                .prg_banks
                .get(self.prg_bank)
                .map(|bank| bank.read(addr)),
            (0xC000..=0xFFFF, 3) => self.prg_banks.last().map(|bank| bank.read(addr)),
            _ => {
                debug!("MMC1 read at {:#06x} not handled", addr);
                None
            }
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> bool {
        match (addr, data, self.shift_count) {
            (0x6000..=0x7FFF, _, _) => {
                if let Some(prg_ram) = self.vram.as_mut() {
                    prg_ram.write(addr, data);
                    true
                } else {
                    false
                }
            }
            (0x8000..=0xFFFF, data, _) if data & 0x80 != 0 => {
                self.control_register.reset();
                self.shift_register = 0;
                self.shift_count = 0;
                true
            }
            (0x8000..=0xFFFF, data, shift_count) if shift_count < 4 => {
                self.shift_register = self.shift_register >> 1 | ((data & 0x01) << 4);
                self.shift_count += 1;
                true
            }
            (0x8000..=0x9FFF, _, _) => {
                self.control_register = ControlRegister(self.load_shift_register(data));
                true
            }
            (0xA000..=0xBFFF, _, _) => {
                self.chr_bank_lo = (self.load_shift_register(data) & 0x1F) as usize;
                true
            }
            (0xC000..=0xDFFF, _, _) => {
                self.chr_bank_hi = (self.load_shift_register(data) & 0x1F) as usize;
                true
            }
            (0xE000..=0xFFFF, _, _) => {
                self.prg_bank = (self.load_shift_register(data) & 0x0F) as usize;
                true
            }
            _ => false,
        }
    }

    fn ppu_map_read(&self, addr: u16) -> Option<u8> {
        match (addr, self.control_register.chr_mode()) {
            (0x0000..=0x1FFF, 0) => self
                .chr_banks
                .get(self.chr_bank_lo)
                .map(|bank| bank.read(addr)),
            (0x0000..=0x0FFF, 1) => self
                .chr_banks
                .get(self.chr_bank_lo & 0xFE)
                .map(|bank| bank.read(addr)),
            (0x1000..=0x1FFF, 1) => self
                .chr_banks
                .get(self.chr_bank_hi & 0xFE)
                .map(|bank| bank.read(addr)),
            _ => None,
        }
    }

    fn ppu_map_write(&mut self, _addr: u16, _data: u8) -> bool {
        false
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(match self.control_register.mirroring() {
            0x00 => Mirroring::OneScreenLo,
            0x01 => Mirroring::OneScreenHi,
            0x02 => Mirroring::Vertical,
            0x03 => Mirroring::Horizontal,
            _ => unreachable!(),
        })
    }

    fn ui(&self, ui: &mut bevy_egui::egui::Ui) {
        ui.monospace(format!("shift register : {:#07b}", self.shift_register));
        ui.monospace(format!("shift count    : {}", self.shift_count));
        ui.monospace(format!(
            "prg mode       : {}",
            self.control_register.prg_mode()
        ));
        ui.monospace(format!("prg selected   : {}", self.prg_bank));
    }
}
