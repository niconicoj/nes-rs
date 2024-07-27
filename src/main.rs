use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_pixel_buffer::pixel_buffer::PixelBufferPlugins;
use nes::NesPlugin;

mod cartridge;
mod cpu;
mod cpu_bus;
mod mem;
mod nes;
mod ppu;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugins))
        .add_plugins(EguiPlugin)
        .add_plugins(NesPlugin)
        .run();
}
