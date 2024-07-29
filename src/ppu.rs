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

bitfield! {
    pub struct LoopyRegister(u16);
    impl Debug;
    pub coarse_x, set_coarse_x: 4, 0;
    pub coarse_y, set_coarse_y: 9, 5;
    pub nametable_x, set_nametable_x: 10, 10;
    pub nametable_y, set_nametable_y: 11, 11;
    pub fine_y, set_fine_y: 14, 12;
}

impl Default for LoopyRegister {
    fn default() -> Self {
        Self(0x0000)
    }
}

#[derive(Component)]
pub struct Ppu {
    pub screen_buffer: [[u8; 256]; 240],
    registers: PpuRegisters,
    name_table: [Mem<0x400>; 2],
    palette_table: [u8; 0x20],
    cycle: i16,
    scanline: i16,
    frame_complete: bool,
    data_buffer: u8,
    addr_latch: bool,
    nmi: bool,
    vram_addr: LoopyRegister,
    tram_addr: LoopyRegister,
    fine_x: u8,
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,
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
            addr_latch: false,
            nmi: false,
            vram_addr: LoopyRegister::default(),
            tram_addr: LoopyRegister::default(),
            fine_x: 0,
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
        }
    }
}

impl Ppu {
    fn increment_scroll_x(&mut self) {
        if self.registers.mask.render_background() || self.registers.mask.render_sprites() {
            if self.vram_addr.coarse_x() == 31 {
                self.vram_addr.set_coarse_x(0);
                let flipped_ntx = !self.vram_addr.nametable_x();
                self.vram_addr.set_nametable_x(flipped_ntx);
            } else {
                let incr_coarse_x = self.vram_addr.coarse_x() + 1;
                self.vram_addr.set_coarse_x(incr_coarse_x);
            }
        }
    }

    fn increment_scroll_y(&mut self) {
        if self.registers.mask.render_background() || self.registers.mask.render_sprites() {
            if self.vram_addr.fine_y() < 7 {
                let incr_fine_y = self.vram_addr.fine_y() + 1;
                self.vram_addr.set_fine_y(incr_fine_y);
            } else {
                self.vram_addr.set_fine_y(0);
                if self.vram_addr.coarse_y() == 29 {
                    self.vram_addr.set_coarse_y(0);
                    let flipped_nty = !self.vram_addr.nametable_y();
                    self.vram_addr.set_nametable_y(flipped_nty);
                } else if self.vram_addr.coarse_y() == 31 {
                    self.vram_addr.set_coarse_y(0);
                } else {
                    let incr_coarse_y = self.vram_addr.coarse_y() + 1;
                    self.vram_addr.set_coarse_y(incr_coarse_y);
                }
            }
        }
    }

    fn transfer_addr_x(&mut self) {
        if self.registers.mask.render_background() || self.registers.mask.render_sprites() {
            self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
            self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
        }
    }

    fn transfer_addr_y(&mut self) {
        if self.registers.mask.render_background() || self.registers.mask.render_sprites() {
            self.vram_addr.set_fine_y(self.tram_addr.fine_y());
            self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
            self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
        }
    }

    fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo =
            (self.bg_shifter_pattern_lo & 0xFF00) | (self.bg_next_tile_lsb as u16);
        self.bg_shifter_pattern_hi =
            (self.bg_shifter_pattern_hi & 0xFF00) | (self.bg_next_tile_msb as u16);

        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00)
            | (((self.bg_next_tile_attrib as u16) & 0b01) * 0xFF);
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00)
            | ((((self.bg_next_tile_attrib as u16) & 0b10) >> 1) * 0xFF);
    }

    fn update_shifters(&mut self) {
        if self.registers.mask.render_background() {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;
            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
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
        if self.ppu.scanline >= -1 && self.ppu.scanline < 240 {
            if self.ppu.scanline == 0 && self.ppu.cycle == 0 {
                self.ppu.cycle = 1;
            }
            if self.ppu.scanline == -1 && self.ppu.cycle == 1 {
                self.ppu.registers.status.set_vblank(false);
            }
            if (self.ppu.cycle >= 2 && self.ppu.cycle < 258)
                || (self.ppu.cycle >= 321 && self.ppu.cycle < 338)
            {
                self.ppu.update_shifters();
                match (self.ppu.cycle - 1) % 8 {
                    0x00 => {
                        self.ppu.load_background_shifters();
                        self.ppu.bg_next_tile_id =
                            self.ppu_read(0x2000 | (self.ppu.vram_addr.0 & 0x0FFF));
                    }
                    0x02 => {
                        self.ppu.bg_next_tile_attrib = self.ppu_read(
                            0x23C0
                                | (self.ppu.vram_addr.nametable_y() << 11)
                                | (self.ppu.vram_addr.nametable_x() << 10)
                                | ((self.ppu.vram_addr.coarse_y() >> 2) << 3)
                                | (self.ppu.vram_addr.coarse_x() >> 2),
                        );
                        if self.ppu.vram_addr.coarse_y() & 0x02 != 0 {
                            self.ppu.bg_next_tile_attrib >>= 4;
                        }
                        if self.ppu.vram_addr.coarse_x() & 0x02 != 0 {
                            self.ppu.bg_next_tile_attrib >>= 2;
                        }
                        self.ppu.bg_next_tile_attrib &= 0x03;
                    }
                    0x04 => {
                        self.ppu.bg_next_tile_lsb = self.ppu_read(
                            ((self.ppu.registers.ctrl.pattern_background() as u16) << 12)
                                + ((self.ppu.bg_next_tile_id as u16) << 4)
                                + (self.ppu.vram_addr.fine_y()),
                        );
                    }
                    0x06 => {
                        self.ppu.bg_next_tile_msb = self.ppu_read(
                            ((self.ppu.registers.ctrl.pattern_background() as u16) << 12)
                                + ((self.ppu.bg_next_tile_id as u16) << 4)
                                + (self.ppu.vram_addr.fine_y())
                                + 8,
                        );
                    }
                    0x07 => {
                        self.ppu.increment_scroll_x();
                    }
                    _ => {}
                }
            }
            if self.ppu.cycle == 256 {
                self.ppu.increment_scroll_y();
            }
            if self.ppu.cycle == 257 {
                self.ppu.load_background_shifters();
                self.ppu.transfer_addr_x();
            }
            if self.ppu.cycle == 338 || self.ppu.cycle == 340 {
                self.ppu.bg_next_tile_id = self.ppu_read(0x2000 | (self.ppu.vram_addr.0 & 0x0FFF));
            }
            if self.ppu.scanline == -1 && self.ppu.cycle >= 280 && self.ppu.cycle < 305 {
                self.ppu.transfer_addr_y();
            }
        }

        if self.ppu.scanline == 240 {}

        if self.ppu.scanline == 241 && self.ppu.cycle == 1 {
            self.ppu.registers.status.set_vblank(true);
            if self.ppu.registers.ctrl.nmi() {
                self.ppu.nmi = true;
            }
        }

        let (bg_pixel, bg_palette) = if self.ppu.registers.mask.render_background() {
            let bit_mux = 0x8000 >> self.ppu.fine_x;

            let px0 = ((self.ppu.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let px1 = ((self.ppu.bg_shifter_pattern_hi & bit_mux) > 0) as u8;
            let bg_pixel = px0 | (px1 << 1);

            let pl0 = ((self.ppu.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let pl1 = ((self.ppu.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            let bg_palette = pl0 | (pl1 << 1);
            (bg_pixel, bg_palette)
        } else {
            (0x00, 0x00)
        };

        let color = self.get_color_from_ram(bg_palette, bg_pixel);
        self.set_pixel(self.ppu.cycle.wrapping_sub(1), self.ppu.scanline, color);

        if self.ppu.frame_complete {
            self.ppu.frame_complete = false;
        }

        self.ppu.cycle = self.ppu.cycle.wrapping_add(1);
        if self.ppu.cycle >= 341 {
            self.ppu.cycle = 0;
            self.ppu.scanline = self.ppu.scanline.wrapping_add(1);
            if self.ppu.scanline >= 261 {
                self.ppu.scanline = -1;
                self.ppu.frame_complete = true;
            }
        }
    }

    fn get_color_from_ram(&mut self, palette: u8, pixel: u8) -> u8 {
        let addr = 0x3F00 + ((palette as u16) << 2) + (pixel as u16);
        self.ppu_read(addr)
    }

    fn set_pixel(&mut self, x: i16, y: i16, color: u8) {
        if let Some(row) = self.ppu.screen_buffer.get_mut(y as usize) {
            if let Some(pixel) = row.get_mut(x as usize) {
                *pixel = color;
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
                let data = if self.ppu.vram_addr.0 >= 0x3F00 {
                    self.ppu.data_buffer = self.ppu_read(self.ppu.vram_addr.0);
                    self.ppu.data_buffer
                } else {
                    let data = self.ppu.data_buffer;
                    self.ppu.data_buffer = self.ppu_read(self.ppu.vram_addr.0);
                    data
                };
                self.ppu.vram_addr.0 += if self.ppu.registers.ctrl.increment_mode() {
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
            0x00 => {
                self.ppu.registers.ctrl.0 = data;
                let ntx = self.ppu.registers.ctrl.nametable_x() as u16;
                self.ppu.tram_addr.set_nametable_x(ntx);
            }
            0x01 => self.ppu.registers.mask.0 = data,
            0x02 => {}
            0x03 => self.ppu.registers.oam_addr = data,
            0x04 => self.ppu.registers.oam_data = data,
            0x05 => {
                if !self.ppu.addr_latch {
                    self.ppu.fine_x = data & 0x07;
                    self.ppu.tram_addr.set_coarse_x((data >> 3) as u16);
                    self.ppu.addr_latch = true;
                } else {
                    self.ppu.tram_addr.set_fine_y((data & 0x07) as u16);
                    self.ppu.tram_addr.set_coarse_y((data >> 3) as u16);
                    self.ppu.addr_latch = false;
                }
            }
            0x06 => {
                if !self.ppu.addr_latch {
                    self.ppu.tram_addr.0 = (self.ppu.tram_addr.0 & 0x00FF) | ((data as u16) << 8);
                    self.ppu.addr_latch = true;
                } else {
                    self.ppu.tram_addr.0 = (self.ppu.tram_addr.0 & 0xFF00) | data as u16;
                    self.ppu.vram_addr.0 = self.ppu.tram_addr.0;
                    self.ppu.addr_latch = false;
                }
            }
            0x07 => {
                self.ppu_write(self.ppu.vram_addr.0, data);
                self.ppu.vram_addr.0 += if self.ppu.registers.ctrl.increment_mode() {
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
                        match self
                            .cartridge
                            .as_ref()
                            .map(|cartridge| cartridge.mirroring())
                        {
                            Some(Mirroring::Vertical) => {
                                let bank_nbr = (addr & 0x0400 >> 10) as usize;
                                self.ppu.name_table[bank_nbr].write(addr, data)
                            }
                            Some(Mirroring::Horizontal) | None => {
                                let bank_nbr = (addr & 0x0800 >> 11) as usize;
                                self.ppu.name_table[bank_nbr].write(addr, data)
                            }
                        }
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
            ui.monospace(format!("PPUADDR:    {a:#06X}", a = query.ppu.vram_addr.0));
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

#[cfg(test)]
mod tests {
    use super::LoopyRegister;

    #[test]
    fn loopy_registers() {
        let mut register = LoopyRegister::default();
        assert_eq!(register.0, 0x0000);

        register.set_nametable_x(true as u16);
        assert_eq!(register.0, 0x0400);

        register.set_coarse_x(0x1F);
        assert_eq!(register.0, 0x041F);

        register.set_coarse_x(0xFF);
        assert_eq!(register.0, 0x041F);

        register.set_coarse_y(0x0E);
        assert_eq!(register.0, 0x05DF);
    }
}
