use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_egui::{egui, EguiContexts};

use crate::{
    cartridge::cartridge_gui,
    cpu::{cpu_gui, disassembly_gui},
    cpu_bus::wram_gui,
    ppu::{draw_pattern_buffer, init_pattern_buffer, pattern_gui, ppu_gui, update_pattern_buffer},
};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuiState>()
            .add_systems(Startup, init_pattern_buffer)
            .add_systems(
                Update,
                (
                    side_panel,
                    cpu_gui.run_if(cpu_gui_enabled),
                    disassembly_gui.run_if(disassembly_gui_enabled),
                    wram_gui.run_if(wram_gui_enabled),
                    cartridge_gui.run_if(cartridge_gui_enabled),
                    ppu_gui.run_if(ppu_gui_enabled),
                    pattern_gui.run_if(pattern_gui_enabled),
                )
                    .run_if(input_toggle_active(false, KeyCode::KeyU)),
            )
            .add_systems(
                PostUpdate,
                (
                    update_pattern_buffer,
                    draw_pattern_buffer.after(update_pattern_buffer),
                )
                    .run_if(pattern_gui_enabled),
            );
    }
}

#[derive(Resource, Default)]
pub struct GuiState {
    cpu: bool,
    wram: bool,
    disassembly: bool,
    cartridge: bool,
    ppu: bool,
    pattern: bool,
}

fn cpu_gui_enabled(state: Res<GuiState>) -> bool {
    state.cpu
}

fn wram_gui_enabled(state: Res<GuiState>) -> bool {
    state.wram
}

fn disassembly_gui_enabled(state: Res<GuiState>) -> bool {
    state.disassembly
}

fn cartridge_gui_enabled(state: Res<GuiState>) -> bool {
    state.cartridge
}

fn ppu_gui_enabled(state: Res<GuiState>) -> bool {
    state.ppu
}

fn pattern_gui_enabled(state: Res<GuiState>) -> bool {
    state.pattern
}

fn side_panel(mut contexts: EguiContexts, mut state: ResMut<GuiState>) {
    egui::SidePanel::right("nes_rs_panel")
        .resizable(false)
        .default_width(150.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| ui.heading("Tools"));
            ui.separator();
            ui.vertical_centered_justified(|ui| {
                if ui.selectable_label(state.cpu, "CPU").clicked() {
                    state.cpu = !state.cpu;
                }
                if ui.selectable_label(state.wram, "WRAM").clicked() {
                    state.wram = !state.wram;
                }
                if ui
                    .selectable_label(state.disassembly, "Disassembly")
                    .clicked()
                {
                    state.disassembly = !state.disassembly;
                }
                if ui.selectable_label(state.cartridge, "Cartridge").clicked() {
                    state.cartridge = !state.cartridge;
                }
                if ui.selectable_label(state.ppu, "PPU").clicked() {
                    state.ppu = !state.ppu;
                }
                if ui
                    .selectable_label(state.pattern, "Pattern table")
                    .clicked()
                {
                    state.pattern = !state.pattern;
                }
            });
        });
}
