use crate::ppu::palette::PaletteState;
use crate::ppu::PpuQuery;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_pixel_buffer::frame::GetFrameFromImages;
use bevy_pixel_buffer::pixel_buffer::PixelBufferSize;
use bevy_pixel_buffer::{builder::PixelBufferBuilder, egui::EguiTexture};

use super::palette::{Palette, PaletteLoader};

const PATTERN_WIDTH: u32 = 128;
const PATTERN_HEIGHT: u32 = 128;

const PATTERN_SIZE: PixelBufferSize = PixelBufferSize {
    size: UVec2::new(PATTERN_WIDTH, PATTERN_HEIGHT),
    pixel_size: UVec2::new(2, 2),
};

#[derive(Component)]
struct PatternBuffer {
    table_id: u16,
    buffer: [u8; 0x4000],
}

impl PatternBuffer {
    fn new(table_id: u16) -> Self {
        Self {
            table_id,
            buffer: [0; 0x4000],
        }
    }
}

pub struct PatternBufferPlugin;

impl Plugin for PatternBufferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaletteState>()
            .init_asset::<Palette>()
            .init_asset_loader::<PaletteLoader>()
            .add_systems(Startup, (init_pattern_buffer, palette_setup))
            .add_systems(
                Update,
                egui_pattern_buffer.run_if(input_toggle_active(false, KeyCode::KeyP)),
            )
            .add_systems(
                PostUpdate,
                update_pattern_buffer.run_if(input_toggle_active(false, KeyCode::KeyP)),
            )
            .add_systems(
                PostUpdate,
                draw_pattern_buffer
                    .after(update_pattern_buffer)
                    .run_if(input_toggle_active(false, KeyCode::KeyP)),
            );
    }
}

fn init_pattern_buffer(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    PixelBufferBuilder::new()
        .with_render(false)
        .with_size(PATTERN_SIZE)
        .spawn(&mut commands, &mut images)
        .entity()
        .insert(PatternBuffer::new(0));

    PixelBufferBuilder::new()
        .with_render(false)
        .with_size(PATTERN_SIZE)
        .spawn(&mut commands, &mut images)
        .entity()
        .insert(PatternBuffer::new(1));
}

fn palette_setup(mut state: ResMut<PaletteState>, asset_server: Res<AssetServer>) {
    state.palette_handle = asset_server.load("palettes/nespalette.pal");
}

fn update_pattern_buffer(ppu: Query<PpuQuery>, mut patterns: Query<&mut PatternBuffer>) {
    if let Ok(ppu) = ppu.get_single() {
        for mut pattern in &mut patterns {
            for addr in 0x00..=0xFF {
                let addr = addr | (pattern.table_id << 8);
                let tile_y = (addr & 0x00F0) >> 1;
                let tile_x = (addr & 0x000F) << 3;
                for offset_y in 0..8 {
                    let mut lsb = ppu.ppu_read((addr << 4) + offset_y);
                    let mut msb = ppu.ppu_read((addr << 4) + offset_y + 8);
                    for offset_x in 0..8 {
                        let bit = (lsb & 0x01) + (msb & 0x01);
                        lsb >>= 1;
                        msb >>= 1;
                        let x = tile_x + (0x07 - offset_x);
                        let y = (tile_y + offset_y) * (PATTERN_WIDTH as u16);
                        pattern.buffer[(x + y) as usize] = bit;
                    }
                }
            }
        }
    }
}

fn draw_pattern_buffer(
    mut images: ResMut<Assets<Image>>,
    palette_state: Res<PaletteState>,
    palettes: Res<Assets<Palette>>,
    pbs: Query<(&Handle<Image>, &PatternBuffer)>,
    query: Query<PpuQuery>,
) {
    if let (Some(palette), Ok(query)) = (
        palettes.get(&palette_state.palette_handle),
        query.get_single(),
    ) {
        for (img, pb) in &pbs {
            images.frame(img).per_pixel(|coord, _| {
                let pixel = pb.buffer[(coord.x + coord.y * PATTERN_WIDTH) as usize] as u16;
                let color_id = query.ppu_read(0x3F00 + (palette_state.palette_id << 2) + pixel);
                palette.get_color(color_id)
            });
        }
    }
}

fn egui_pattern_buffer(
    mut contexts: EguiContexts,
    pbs: Query<(&EguiTexture, &PatternBuffer)>,
    mut state: ResMut<PaletteState>,
) {
    // let (texture, _) = pbs.single();
    egui::Window::new("Pattern Buffer").show(&contexts.ctx_mut(), |ui| {
        egui::ComboBox::from_label("Palette ID")
            .selected_text(format!("{:?}", state.palette_id))
            .show_ui(ui, |ui| {
                (0..8).for_each(|palette_id| {
                    ui.selectable_value(
                        &mut state.palette_id,
                        palette_id,
                        format!("{:?}", palette_id),
                    );
                });
            });
        ui.horizontal(|ui| {
            for (texture, _) in &pbs {
                ui.image(egui::load::SizedTexture::new(texture.id, texture.size));
            }
        });
    });
}
