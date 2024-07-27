use bevy_egui::egui::{self, ScrollArea};

use crate::mem::Mem;

use super::mapper::Mapper;

#[derive(Default)]
pub struct Nrom128 {
    prg_rom: Mem<0x4000>,
    chr_rom: Mem<0x2000>,
}

impl Nrom128 {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        let mut nrom = Self::default();
        nrom.prg_rom.write_slice(0, &prg_rom);
        nrom.chr_rom.write_slice(0, &chr_rom);
        nrom
    }
}

impl Mapper for Nrom128 {
    fn cpu_read(&self, addr: u16) -> Option<u8> {
        if addr >= 0x8000 {
            Some(self.prg_rom.read(addr))
        } else {
            None
        }
    }

    fn cpu_write(&mut self, _addr: u16, _data: u8) -> Option<()> {
        Some(())
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        if addr < 0x2000 {
            Some(self.chr_rom.read(addr))
        } else {
            None
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) -> Option<()> {
        match addr {
            0x0000..=0x1FFF => self.chr_rom.write(addr, data),
            _ => return None,
        };
        Some(())
    }

    fn ui(&self, ui: &mut egui::Ui) {
        ui.label("NROM-128");

        let text_style = egui::TextStyle::Monospace;
        let row_height = ui.text_style_height(&text_style);
        let total_rows = 0x8000 / 16;
        ui.separator();
        ui.label("PRG Memory");
        ui.separator();

        ui.push_id("prg", |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .max_height(200.0)
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for row in row_range {
                        let start = 0x8000 + (row * 16) as u16;
                        let end = start.saturating_add(16);
                        let row_text = (start..end)
                            .map(|addr| format!("{:02X}", self.cpu_read(addr).unwrap_or(0)))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(format!("${:#06X}: {}", start, row_text));
                    }
                });
        });
        ui.label("CHR Memory");
        ui.separator();

        let row_height = ui.text_style_height(&text_style);
        let total_rows = 0x2000 / 16;
        ui.push_id("chr", |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .max_height(200.0)
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for row in row_range {
                        let start = 0x0000 + (row * 16) as u16;
                        let end = start.saturating_add(16);
                        let row_text = (start..end)
                            .map(|addr| format!("{:02X}", self.ppu_read(addr).unwrap_or(0)))
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(format!("${:#06X}: {}", start, row_text));
                    }
                });
        });
    }
}
