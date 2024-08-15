use bevy::{prelude::*, window::WindowResized};
use bevy_fundsp::prelude::Num;
use bevy_pixel_buffer::{
    builder::PixelBufferBuilder,
    frame::GetFrameFromImages,
    pixel_buffer::{PixelBuffer, PixelBufferSize},
};

use super::{
    palette::{Palette, PaletteState},
    Ppu,
};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

const SCREEN_SIZE: PixelBufferSize = PixelBufferSize {
    size: UVec2::new(SCREEN_WIDTH, SCREEN_HEIGHT),
    pixel_size: UVec2::new(1, 1),
};

#[derive(Component)]
struct ScreenBuffer;

pub struct ScreenBufferPlugin;

impl Plugin for ScreenBufferPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_screen_buffer).add_systems(
            PostUpdate,
            (update_screen_buffer, resize_screen_buffer).chain(),
        );
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

fn resize_screen_buffer(
    mut resize_reader: EventReader<WindowResized>,
    mut pb: Query<&mut Transform, With<PixelBuffer>>,
) {
    if let Ok(mut tf) = pb.get_single_mut() {
        for e in resize_reader.read() {
            let px_dim = (e.width / (SCREEN_WIDTH as f32)).min(e.height / (SCREEN_HEIGHT as f32));
            info!("px_dim: {}", px_dim);
            tf.scale = Vec3::new(px_dim, px_dim, 1.0);
        }
    }
}
