use bevy::prelude::*;

use crate::{
    cartridge::CartridgePlugin,
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

pub struct NesPlugin;

impl Plugin for NesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CpuPlugin, CpuBusPlugin, CartridgePlugin, PpuPlugin))
            .add_systems(Startup, init_nes);
    }
}

fn init_nes(mut commands: Commands) {
    commands.spawn(NesBundle::default());
}
