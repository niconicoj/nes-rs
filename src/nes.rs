use bevy::prelude::*;
use clap::Parser;

use crate::{
    apu::{Apu, ApuPlugin},
    cartridge::Cartridge,
    cpu::{Cpu, CpuPlugin, SystemClock},
    cpu_bus::{update_controller_state, Controller, Dma, Wram},
    ppu::{PalettePlugin, Ppu, PpuPlugin},
};

#[derive(Default, Component)]
pub struct NesMarker;

#[derive(Default, Bundle)]
pub struct NesBundle {
    marker: NesMarker,
    system_clock: SystemClock,
    cpu: Cpu,
    dma: Dma,
    wram: Wram,
    ppu: Ppu,
    apu: Apu,
    controller: Controller,
}

#[derive(Parser, Resource, Clone)]
#[command(version, about, long_about = None)]
pub struct ArgsResource {
    #[arg(short, long)]
    /// optional path to a rom file.
    pub rom: Option<String>,
}

pub struct NesPlugin {
    args: ArgsResource,
}

impl NesPlugin {
    pub fn new(args: ArgsResource) -> Self {
        Self { args }
    }
}

impl Plugin for NesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.args.clone())
            .add_plugins((CpuPlugin, PpuPlugin, PalettePlugin, ApuPlugin))
            .add_systems(Startup, init_nes)
            .add_systems(Update, update_controller_state);
    }
}

fn init_nes(mut commands: Commands, args: Res<ArgsResource>) {
    match &args.rom {
        Some(rom_path) => {
            let cartridge = Cartridge::from_file(&rom_path)
                .expect("Rom path should point to a valid rom file.");
            info!("Loaded rom: {}", rom_path);
            commands.spawn((NesBundle::default(), cartridge));
        }
        None => {
            commands.spawn(NesBundle::default());
        }
    }
}
