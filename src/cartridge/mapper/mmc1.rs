use std::io::BufRead;

use bevy::log::info;
use bevy_egui::egui::{self, ScrollArea, Separator};
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

    let chr_ram = if header.chr_rom_banks == 0 {
        info!("using CHR RAM");
        Some(Mem::default())
    } else {
        info!("No CHR RAM");
        None
    };

    Box::new(Mmc1::new(prg_banks, chr_banks, chr_ram, header.mirroring))
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
    prg_ram: Mem<0x2000>,
    chr_ram: Option<Mem<0x4000>>,
    prg_banks: Vec<Mem<0x4000>>,
    chr_banks: Vec<Mem<0x2000>>,
}

impl Mmc1 {
    pub fn new(
        prg_rom_banks: Vec<Mem<0x4000>>,
        chr_rom_banks: Vec<Mem<0x2000>>,
        chr_ram: Option<Mem<0x4000>>,
        mirroring: Mirroring,
    ) -> Self {
        Self {
            control_register: ControlRegister(0x1C | mirroring as u8),
            shift_register: 0,
            shift_count: 0,
            chr_bank_hi: 0,
            chr_bank_lo: 0,
            prg_bank: 0,
            prg_ram: Mem::default(),
            chr_ram,
            prg_banks: prg_rom_banks,
            chr_banks: chr_rom_banks,
        }
    }
}

impl Mmc1 {
    fn load_shift_register(&mut self, data: u8) -> u8 {
        let loaded_value = (self.shift_register >> 1) | ((data & 0x01) << 4);
        self.shift_count = 0;
        self.shift_register = 0;
        loaded_value
    }
}

impl Mapper for Mmc1 {
    fn cpu_map_read(&self, addr: u16) -> Option<u8> {
        match (addr, self.control_register.prg_mode()) {
            (0x6000..=0x7FFF, _) => Some(self.prg_ram.read(addr)),
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
            _ => None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> bool {
        match (addr, data, self.shift_count) {
            (0x6000..=0x7FFF, _, _) => {
                self.prg_ram.write(addr, data);
                true
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
        if let Some(chr_ram) = self.chr_ram.as_ref() {
            if addr < 0x2000 {
                Some(chr_ram.read(addr))
            } else {
                None
            }
        } else {
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
    }

    fn ppu_map_write(&mut self, addr: u16, data: u8) -> bool {
        if addr < 0x2000 {
            if let Some(chr_ram) = self.chr_ram.as_mut() {
                chr_ram.write(addr, data);
                true
            } else {
                false
            }
        } else {
            false
        }
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

        ui.label("PRG banks");
        ui.monospace("         0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");
        ui.add(Separator::default().spacing(2.0));
        let text_style = egui::TextStyle::Monospace;
        let row_height = ui.text_style_height(&text_style);
        let total_rows = 0x2000 / 16;
        ui.push_id("prg_memory", |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .max_height(200.0)
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for row in row_range {
                        let start = (0x2000 + row * 16) as u16;
                        let end = start + 16;
                        let row_text = (start..end)
                            .map(|addr| format!("{:02X}", self.prg_ram.read(addr)))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(format!("${:#06X}: {}", start, row_text));
                    }
                });
        });
    }
}
