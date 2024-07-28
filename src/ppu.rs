use bevy::{ecs::query::QueryData, prelude::*};
use bevy_egui::{egui, EguiContexts};
use bitfield::bitfield;
pub use pattern_buffer::{
    draw_pattern_buffer, init_pattern_buffer, pattern_gui, update_pattern_buffer,
};
use screen_buffer::ScreenBufferPlugin;

use crate::{
    cartridge::{Cartridge, Mirroring},
    mem::Mem,
};

mod palette;
mod pattern_buffer;
mod screen_buffer;

pub use palette::PalettePlugin;

#[derive(Debug)]
pub struct PpuRegisters {
    pub ctrl: PpuCtrl,
    pub mask: PpuMask,
    pub status: PpuStatus,
    pub oam_addr: u8,
    pub oam_data: u8,
    pub scroll: u8,
    pub addr: u8,
    pub data: u8,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            ctrl: PpuCtrl(0),
            mask: PpuMask(0),
            status: PpuStatus(0),
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: 0,
            data: 0,
        }
    }
}

bitfield! {
    pub struct PpuCtrl(u8);
    impl Debug;
    pub nametable_x, set_nametable_x: 0;
    pub nametable_y, set_nametable_y: 1;
    pub increment_mode, set_increment_mode: 2;
    pub pattern_sprite, set_pattern_sprite: 3;
    pub pattern_background, set_pattern_background: 4;
    pub slave_mode, set_slave_mode: 5;
    pub nmi, set_nmi: 7;
}

bitfield! {
    pub struct PpuMask(u8);
    impl Debug;
    pub greyscale, set_greyscale: 0;
    pub render_background_left, set_render_background_left: 1;
    pub render_sprite_left, set_render_sprite_left: 2;
    pub render_background, set_render_background: 3;
    pub render_sprites, set_render_sprites: 4;
    pub emphasize_red, set_emphasize_red: 5;
    pub emphasize_green, set_emphasize_green: 6;
    pub emphasize_blue, set_emphasize_blue: 7;
}

bitfield! {
    pub struct PpuStatus(u8);
    impl Debug;
    pub sprite_overflow, set_sprite_overflow: 5;
    pub sprite_zero_hit, set_sprite_zero_hit: 6;
    pub vblank, set_vblank: 7;
}

#[derive(Component)]
pub struct Ppu {
    pub screen_buffer: [[u8; 256]; 240],
    registers: PpuRegisters,
    name_table: [Mem<0x400>; 2],
    palette_table: [u8; 0x20],
    cycle: u16,
    scanline: u16,
    frame_complete: bool,
    data_buffer: u8,
    ppu_addr: u16,
    addr_latch: bool,
    nmi: bool,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            screen_buffer: [[0; 256]; 240],
            registers: PpuRegisters::default(),
            name_table: [Mem::<0x400>::default(), Mem::<0x400>::default()],
            palette_table: [0; 0x20],
            cycle: 0,
            scanline: 0,
            frame_complete: false,
            data_buffer: 0x00,
            ppu_addr: 0x00,
            addr_latch: false,
            nmi: false,
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct PpuQuery {
    ppu: &'static mut Ppu,
    cartridge: Option<&'static mut Cartridge>,
}

impl<'w> PpuQueryItem<'w> {
    pub fn reset(&mut self) {
        self.ppu.registers.ctrl.0 = 0x00;
        self.ppu.registers.mask.0 = 0x00;
        self.ppu.registers.scroll = 0x00;
        self.ppu.data_buffer = 0x00;
        self.ppu.addr_latch = false;
    }

    pub fn frame_complete(&mut self) -> bool {
        if self.ppu.frame_complete {
            self.ppu.frame_complete = false;
            true
        } else {
            false
        }
    }

    pub fn nmi(&mut self) -> bool {
        let v = self.ppu.nmi;
        self.ppu.nmi = false;
        v
    }

    pub fn tick(&mut self) {
        if self.ppu.scanline == u16::MAX && self.ppu.cycle == 1 {
            self.ppu.registers.status.set_vblank(false);
        }
        if self.ppu.scanline == 241 && self.ppu.cycle == 1 {
            self.ppu.registers.status.set_vblank(true);
            if self.ppu.registers.ctrl.nmi() {
                self.ppu.nmi = true;
            }
        }
        if self.ppu.frame_complete {
            self.ppu.frame_complete = false;
        }

        self.ppu.cycle = self.ppu.cycle.wrapping_add(1);
        if self.ppu.cycle >= 341 {
            self.ppu.cycle = 0;
            self.ppu.scanline = self.ppu.scanline.wrapping_add(1);
            if self.ppu.scanline >= 261 {
                self.ppu.scanline = u16::MAX;
                self.ppu.frame_complete = true;
            }
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x2000..=0x3FFF => Some(self.ppu_register_read(addr)),
            0x4020..=0xFFFF => self
                .cartridge
                .as_mut()
                .and_then(|cartridge| cartridge.cpu_read(addr)),
            _ => None,
        }
    }
    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000..=0x3FFF => self.ppu_register_write(addr, data),
            0x4020..=0xFFFF => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.cpu_write(addr, data);
                }
            }
            _ => {}
        }
    }

    fn ppu_register_read(&mut self, addr: u16) -> u8 {
        let addr = addr & 0x07;
        match addr {
            0x00 => self.ppu.registers.ctrl.0,
            0x01 => self.ppu.registers.mask.0,
            0x02 => {
                let data = (self.ppu.registers.status.0 & 0xE0) | (self.ppu.data_buffer & 0x1F);
                self.ppu.registers.status.set_vblank(false);
                self.ppu.addr_latch = false;
                data
            }
            0x03 => self.ppu.registers.oam_addr,
            0x04 => self.ppu.registers.oam_data,
            0x05 => self.ppu.registers.scroll,
            0x06 => self.ppu.registers.addr,
            0x07 => {
                let data = if self.ppu.ppu_addr >= 0x3F00 {
                    self.ppu.data_buffer = self.ppu_read(self.ppu.ppu_addr);
                    self.ppu.data_buffer
                } else {
                    let data = self.ppu.data_buffer;
                    self.ppu.data_buffer = self.ppu_read(self.ppu.ppu_addr);
                    data
                };
                self.ppu.ppu_addr += if self.ppu.registers.ctrl.increment_mode() {
                    32
                } else {
                    1
                };
                data
            }
            _ => unreachable!(),
        }
    }

    fn ppu_register_write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x07;
        match addr {
            0x00 => self.ppu.registers.ctrl.0 = data,
            0x01 => self.ppu.registers.mask.0 = data,
            0x02 => {}
            0x03 => self.ppu.registers.oam_addr = data,
            0x04 => self.ppu.registers.oam_data = data,
            0x05 => self.ppu.registers.scroll = data,
            0x06 => {
                if !self.ppu.addr_latch {
                    self.ppu.ppu_addr = (self.ppu.ppu_addr & 0x00FF) | ((data as u16) << 8);
                    self.ppu.addr_latch = true;
                } else {
                    self.ppu.ppu_addr = (self.ppu.ppu_addr & 0xFF00) | data as u16;
                    self.ppu.addr_latch = false;
                }
            }
            0x07 => {
                self.ppu_write(self.ppu.ppu_addr, data);
                self.ppu.ppu_addr += if self.ppu.registers.ctrl.increment_mode() {
                    32
                } else {
                    1
                };
            }
            _ => unreachable!(),
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self
                .cartridge
                .as_ref()
                .and_then(|cartridge| cartridge.ppu_read(addr))
                .unwrap_or(0),
            0x2000..=0x3EFF => {
                match self
                    .cartridge
                    .as_ref()
                    .and_then(|mapper| mapper.ppu_read(addr))
                {
                    Some(data) => data,
                    None => match self
                        .cartridge
                        .as_ref()
                        .map(|cartridge| cartridge.mirroring())
                    {
                        Some(Mirroring::Vertical) => {
                            let bank_nbr = (addr & 0x0400 >> 10) as usize;
                            self.ppu.name_table[bank_nbr].read(addr)
                        }
                        Some(Mirroring::Horizontal) | None => {
                            let bank_nbr = (addr & 0x0800 >> 11) as usize;
                            self.ppu.name_table[bank_nbr].read(addr)
                        }
                    },
                }
            }
            0x3F00..=0x3FFF => {
                let addr = addr & 0x1F;
                let addr = match addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1C => 0x0C,
                    _ => addr,
                };
                self.ppu.palette_table[addr as usize]
            }
            _ => 0,
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if let Some(cartridge) = &mut self.cartridge {
                    let _ = cartridge.ppu_write(addr, data);
                }
            }
            0x2000..=0x3EFF => {
                if let Some(mapper) = &mut self.cartridge {
                    if !mapper.ppu_write(addr, data) {
                        let bank_nbr = (addr & 0x0800 >> 11) as usize;
                        self.ppu.name_table[bank_nbr].write(addr, data);
                    }
                }
            }
            0x3F00..=0x3FFF => {
                let addr = addr & 0x1F;
                let addr = match addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1C => 0x0C,
                    _ => addr,
                };
                self.ppu.palette_table[addr as usize] = data;
            }
            _ => {}
        }
    }
}

impl<'w> PpuQueryReadOnlyItem<'w> {
    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3FFF => self.ppu_register_read(addr),
            0x4020..=0xFFFF => self
                .cartridge
                .as_ref()
                .and_then(|cartridge| cartridge.cpu_read(addr))
                .unwrap_or(0),
            _ => 0,
        }
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self
                .cartridge
                .as_ref()
                .and_then(|cartridge| cartridge.ppu_read(addr))
                .unwrap_or(0),
            0x2000..=0x3EFF => self
                .cartridge
                .as_ref()
                .and_then(|mapper| mapper.ppu_read(addr))
                .unwrap_or_else(|| {
                    let bank_nbr = (addr & 0x0800 >> 11) as usize;
                    self.ppu.name_table[bank_nbr].read(addr)
                }),
            0x3F00..=0x3FFF => {
                let addr = addr & 0x1F;
                let addr = match addr {
                    0x10 => 0x00,
                    0x14 => 0x04,
                    0x18 => 0x08,
                    0x1C => 0x0C,
                    _ => addr,
                };
                self.ppu.palette_table[(addr & 0x1F) as usize]
            }
            _ => 0,
        }
    }

    fn ppu_register_read(&self, addr: u16) -> u8 {
        let addr = addr & 0x07;
        match addr {
            0x00 => self.ppu.registers.ctrl.0,
            0x01 => self.ppu.registers.mask.0,
            0x02 => self.ppu.registers.status.0,
            0x03 => self.ppu.registers.oam_addr,
            0x04 => self.ppu.registers.oam_data,
            0x05 => self.ppu.registers.scroll,
            0x06 => self.ppu.registers.addr,
            0x07 => self.ppu.registers.data,
            _ => unreachable!(),
        }
    }
}

pub struct PpuPlugin;

impl Plugin for PpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScreenBufferPlugin);
    }
}

pub fn ppu_gui(query: Query<PpuQuery>, mut contexts: EguiContexts) {
    egui::Window::new("PPU Info").show(&contexts.ctx_mut(), |ui| {
        if let Ok(query) = query.get_single() {
            ui.label("Registers");
            ui.monospace(format!(
                "(0x2000) PPUCTRL:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.ctrl.0
            ));
            ui.monospace(format!(
                "(0x2001) PPUMASK:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.mask.0
            ));
            ui.monospace(format!(
                "(0x2002) PPUSTATUS: {a:#04X} ({a:#010b})",
                a = query.ppu.registers.status.0
            ));
            ui.monospace(format!(
                "(0x2003) OAMADDR:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.oam_addr
            ));
            ui.monospace(format!(
                "(0x2004) OAMDATA:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.oam_data
            ));
            ui.monospace(format!(
                "(0x2005) PPUSCROLL: {a:#04X} ({a:#010b})",
                a = query.ppu.registers.scroll
            ));
            ui.monospace(format!(
                "(0x2006) PPUADDR:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.addr
            ));
            ui.monospace(format!(
                "(0x2007) PPUDATA:   {a:#04X} ({a:#010b})",
                a = query.ppu.registers.data
            ));
            ui.separator();
            ui.label("State");
            ui.monospace(format!("PPUADDR:    {a:#06X}", a = query.ppu.ppu_addr));
            ui.monospace(format!(
                "DATABUFFER: {a:#04X} ({a:#010b})",
                a = query.ppu.data_buffer
            ));
            ui.monospace(format!(
                "ADDRLATCH:  {a:#04X} ({a:#010b})",
                a = query.ppu.addr_latch as u8
            ));
            ui.separator();
            ui.label("Palettes");
            egui::Grid::new("palette_grid")
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.monospace(format!("BG: {:#04x}", query.ppu_read(0x3F00)));
                    ui.end_row();
                    for palette_id in 0..8 {
                        let palette_str = (0..4)
                            .map(|i| {
                                format!("{:#04x}", query.ppu_read(0x3F00 + (palette_id << 2) + i))
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        ui.monospace(format!("P{}: {}", palette_id, palette_str));
                        ui.end_row();
                    }
                });
        } else {
            ui.label("No PPU found");
        }
    });
}
