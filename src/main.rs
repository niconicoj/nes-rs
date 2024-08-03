use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_fundsp::prelude::*;
use bevy_pixel_buffer::pixel_buffer::PixelBufferPlugins;
use clap::Parser;
use gui::GuiPlugin;
use nes::NesPlugin;

mod apu;
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
        .add_plugins((DefaultPlugins, PixelBufferPlugins, EguiPlugin))
        .add_plugins(DspPlugin::default())
        .add_plugins((GuiPlugin, NesPlugin::new(args)))
        .run();
}
