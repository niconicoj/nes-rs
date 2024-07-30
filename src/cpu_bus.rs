use bevy::{ecs::query::QueryData, prelude::*};
use bevy_egui::{
    egui::{self, ScrollArea, Separator},
    EguiContexts,
};

use crate::ppu::PpuQuery;

#[derive(Component, Default)]
pub struct Controller {
    state: u8,
    shifter: u8,
}

impl Controller {
    fn store_shifter(&mut self) {
        self.shifter = self.state;
    }

    fn read_shifter(&mut self) -> u8 {
        let result = (self.shifter & 0x80 != 0) as u8;
        self.shifter <<= 1;
        result
    }
}

pub fn update_controller_state(mut query: Query<&mut Controller>, keys: Res<ButtonInput<KeyCode>>) {
    if let Ok(mut controller) = query.get_single_mut() {
        controller.state = 0x00;
        keys.get_pressed().for_each(|key| match key {
            KeyCode::KeyW => controller.state |= 0x80, // A
            KeyCode::KeyX => controller.state |= 0x40, // B
            KeyCode::KeyQ => controller.state |= 0x20, // select
            KeyCode::KeyS => controller.state |= 0x10, // start
            KeyCode::ArrowUp => controller.state |= 0x08,
            KeyCode::ArrowDown => controller.state |= 0x04,
            KeyCode::ArrowLeft => controller.state |= 0x02,
            KeyCode::ArrowRight => controller.state |= 0x01,
            _ => {}
        });
    }
}

#[derive(Component)]
pub struct Wram {
    data: [u8; 0x800],
}

impl Default for Wram {
    fn default() -> Self {
        Self { data: [0; 0x800] }
    }
}

impl Wram {
    fn read(&self, addr: u16) -> u8 {
        self.data[(addr as usize) % 0x800]
    }
    fn write(&mut self, addr: u16, value: u8) {
        self.data[(addr as usize) % 0x800] = value;
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CpuBusQuery {
    wram: &'static mut Wram,
    controller: &'static mut Controller,
    ppu: PpuQuery,
}

impl<'w> CpuBusQueryReadOnlyItem<'w> {
    pub fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.wram.read(addr),
            0x2000..=0x3FFF => self.ppu.cpu_read(addr),
            0x4020..=0xFFFF => self.ppu.cpu_read(addr),
            _ => 0x00,
        }
    }
}

impl<'w> CpuBusQueryItem<'w> {
    pub fn reset(&mut self) {
        self.ppu.reset();
    }

    pub fn frame_complete(&mut self) -> bool {
        self.ppu.frame_complete()
    }

    pub fn nmi(&mut self) -> bool {
        self.ppu.nmi()
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
    }

    pub fn cpu_read(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x0000..=0x1FFF => Some(self.wram.read(addr)),
            0x2000..=0x3FFF => self.ppu.cpu_read(addr),
            0x4016 | 0x4017 => Some(self.controller.read_shifter()),
            0x4020..=0xFFFF => self.ppu.cpu_read(addr),
            _ => None,
        }
    }
    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.wram.write(addr, data),
            0x2000..=0x3FFF => self.ppu.cpu_write(addr, data),
            0x4016 | 0x4017 => self.controller.store_shifter(),
            0x4020..=0xFFFF => self.ppu.cpu_write(addr, data),
            _ => {}
        };
    }
}

pub fn wram_gui(wram: Query<&Wram>, mut contexts: EguiContexts) {
    let wram = wram.single();
    egui::Window::new("WRAM Info")
        .min_width(420.0)
        .show(&contexts.ctx_mut(), |ui| {
            ui.monospace("         0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");
            ui.add(Separator::default().spacing(2.0));
            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);
            let total_rows = 0x2000 / 16;
            ui.push_id("wram", |ui| {
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(200.0)
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        for row in row_range {
                            let start = (row * 16) as u16;
                            let end = start + 16;
                            let row_text = (start..end)
                                .map(|addr| format!("{:02X}", wram.data[(addr % 0x800) as usize]))
                                .collect::<Vec<_>>()
                                .join(" ");
                            ui.monospace(format!("${:#06X}: {}", start, row_text));
                        }
                    });
            });
        });
}
