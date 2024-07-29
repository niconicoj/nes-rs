use bevy::prelude::*;
use bevy_pixel_buffer::{
    builder::PixelBufferBuilder, frame::GetFrameFromImages, pixel_buffer::PixelBufferSize,
};

use super::{
    palette::{Palette, PaletteState},
    Ppu,
};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

const SCREEN_SIZE: PixelBufferSize = PixelBufferSize {
    size: UVec2::new(SCREEN_WIDTH, SCREEN_HEIGHT),
    pixel_size: UVec2::new(4, 4),
};

#[derive(Component)]
struct ScreenBuffer;

pub struct ScreenBufferPlugin;

impl Plugin for ScreenBufferPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_screen_buffer)
            .add_systems(PostUpdate, update_screen_buffer);
    }
}

fn init_screen_buffer(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    PixelBufferBuilder::new()
        .with_render(true)
        .with_size(SCREEN_SIZE)
        .spawn(&mut commands, &mut images)
        .entity()
        .insert(ScreenBuffer);
}

fn update_screen_buffer(
    mut images: ResMut<Assets<Image>>,
    palette_state: Res<PaletteState>,
    palettes: Res<Assets<Palette>>,
    pb: Query<(&Handle<Image>, &ScreenBuffer)>,
    ppu: Query<&Ppu>,
) {
    if let (Ok((pb, _)), Ok(ppu), Some(palette)) = (
        pb.get_single(),
        ppu.get_single(),
        palettes.get(&palette_state.palette_handle),
    ) {
        images.frame(pb).per_pixel(|coord, _| {
            let color_id = ppu.screen_buffer[coord.y as usize][coord.x as usize];
            palette
                .get_color(color_id)
                .expect(&format!("invalid color id {:#04x}", color_id))
        });
    }
}
