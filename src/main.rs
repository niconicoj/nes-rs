use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_pixel_buffer::pixel_buffer::PixelBufferPlugins;
use clap::Parser;
use gui::GuiPlugin;
use nes::NesPlugin;

mod cartridge;
mod cpu;
mod cpu_bus;
mod gui;
mod mem;
mod nes;
mod ppu;

fn main() {
    let args = nes::ArgsResource::parse();

    App::new()
        .add_plugins((DefaultPlugins, PixelBufferPlugins))
        .add_plugins(EguiPlugin)
        .add_plugins(GuiPlugin)
        .add_plugins(NesPlugin::new(args))
        .run();
}
