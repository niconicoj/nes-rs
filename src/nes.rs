use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_egui::{egui, EguiContexts};
use clap::Parser;

use crate::{
    cartridge::{self, Cartridge, CartridgePlugin},
    cpu::{Cpu, CpuPlugin, SystemClock},
    cpu_bus::{CpuBusPlugin, Wram},
    ppu::{Ppu, PpuPlugin},
};

#[derive(Default, Component)]
pub struct NesMarker;

#[derive(Default, Bundle)]
pub struct NesBundle {
    marker: NesMarker,
    system_clock: SystemClock,
    cpu: Cpu,
    wram: Wram,
    ppu: Ppu,
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
            .add_plugins((CpuPlugin, CpuBusPlugin, CartridgePlugin, PpuPlugin))
            .add_systems(Startup, init_nes)
            .add_systems(
                Update,
                main_ui.run_if(input_toggle_active(false, KeyCode::KeyU)),
            );
    }
}

fn init_nes(mut commands: Commands, args: Res<ArgsResource>) {
    match &args.rom {
        Some(rom_path) => {
            let cartridge = Cartridge::from_file(&rom_path)
                .expect("Rom path should point to a valid rom file.");
            commands.spawn((NesBundle::default(), cartridge));
        }
        None => {
            commands.spawn(NesBundle::default());
        }
    }
}

fn main_ui(mut contexts: EguiContexts) {
    egui::SidePanel::right("nes_rs_panel")
        .resizable(false)
        .default_width(150.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| ui.heading("Tools"));
            ui.separator();
        });
}
