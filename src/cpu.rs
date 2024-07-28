use std::fmt::UpperHex;

use crate::cpu_bus::CpuBusQuery;
use addr_mode::AddrMode;
use bevy::{ecs::query::QueryData, prelude::*, utils::HashSet};
use bevy_egui::{
    egui::{self, Color32, RichText, ScrollArea},
    EguiContexts,
};
use bitfield::bitfield;
use instr::{Instr, INSTRUCTION_TABLE};

mod addr_mode;
mod instr;
mod op;

#[derive(Component)]
pub struct SystemClock {
    enabled: bool,
    pub cycles: usize,
    timer: Timer,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            enabled: false,
            cycles: 0,
            timer: Timer::from_seconds(1.0 / 60.0, TimerMode::Repeating),
        }
    }
}

impl SystemClock {
    fn reset(&mut self) {}
}

bitfield! {
    struct CpuStatus(u8);
    impl Debug;
    carry, set_carry: 0;
    zero, set_zero: 1;
    no_interrupt, set_no_interrupt: 2;
    decimal, set_decimal: 3;
    b_flag, set_b_flag: 4;
    u, _: 5;
    overflow, set_overflow: 6;
    negative, set_negative: 7;
}

impl Default for CpuStatus {
    fn default() -> Self {
        Self(0b0010_0100)
    }
}

#[derive(Component)]
pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    status: CpuStatus,
    cycles: u8,
    open_bus: u8,
    addr_mode: AddrMode,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0xFFFC,
            cycles: 0,
            status: CpuStatus::default(),
            open_bus: 0x00,
            addr_mode: AddrMode::_XXX,
        }
    }
}

impl Cpu {
    /// increment the program counter and return its value before increment
    fn adv(&mut self) -> u16 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(1);
        pc
    }

    fn reset(&mut self, addr: u16) {
        self.pc = addr;
        self.a = 0;
        self.x = 0;
        self.y = 0;

        // really ?
        self.sp = self.sp.wrapping_sub(3).min(0xFF);

        self.cycles = 8;
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CpuQuery {
    cpu: &'static mut Cpu,
    bus: CpuBusQuery,
    clock: &'static mut SystemClock,
}

impl<'w> CpuQueryReadOnlyItem<'w> {
    // produces a disassembly of the code starting at the next intruction
    // and continuing for `count` instructions
    pub fn disassemble(&self, count: u16) -> Vec<String> {
        let mut pc = self.cpu.pc;
        let mut disassembly = Vec::new();
        for _ in 0..count {
            let (instr, len) = self.instr_at(pc);
            disassembly.push(instr);
            pc = pc.wrapping_add(len + 1);
        }
        disassembly
    }

    fn instr_at(&self, addr: u16) -> (String, u16) {
        let opcode = self.bus_read(addr);
        let instr = INSTRUCTION_TABLE[opcode as usize];

        let operand = match instr.addr_mode() {
            AddrMode::IMM
            | AddrMode::ZP0
            | AddrMode::ZPX
            | AddrMode::ZPY
            | AddrMode::REL
            | AddrMode::IDX
            | AddrMode::IDY => Operand::Byte(self.bus.cpu_read(addr.wrapping_add(1))),
            AddrMode::ABS | AddrMode::ABX | AddrMode::ABY | AddrMode::IND => {
                Operand::Word(u16::from_le_bytes([
                    self.bus.cpu_read(addr.wrapping_add(1)),
                    self.bus.cpu_read(addr.wrapping_add(2)),
                ]))
            }

            _ => Operand::Unary,
        };

        let instr = format!(
            "{:#06X}: {:?} {:#06X}:{:?}",
            addr,
            instr.op(),
            operand,
            instr.addr_mode()
        );
        (instr, operand.len())
    }

    fn bus_read(&self, addr: u16) -> u8 {
        self.bus.cpu_read(addr)
    }
}

enum Operand {
    Unary,
    Byte(u8),
    Word(u16),
}

impl Operand {
    fn len(&self) -> u16 {
        match self {
            Operand::Byte(_) => 1,
            Operand::Word(_) => 2,
            Operand::Unary => 0,
        }
    }
}

impl UpperHex for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operand::Byte(val) => format!("{:#06X}", val),
                Operand::Word(val) => format!("{:#06X}", val),
                Operand::Unary => "".to_string(),
            }
        )
    }
}

impl<'w> CpuQueryItem<'w> {
    pub fn next_frame(&mut self, breakpoints: Option<&BreakPointState>) -> bool {
        while !self.bus.frame_complete() {
            self.clock();
            if let Some(breakpoints) = breakpoints {
                if breakpoints.check(self.cpu.pc) {
                    return true;
                }
            }
        }
        return false;
    }
    pub fn clock(&mut self) {
        self.clock.cycles += 1;
        self.bus.tick();
        if self.clock.cycles % 3 == 0 {
            self.tick();
        }
        if self.bus.nmi() {
            self.nmi();
        }
    }

    fn tick(&mut self) {
        if self.cpu.cycles == 0 {
            let op_addr = self.cpu.adv();
            let opcode = self.bus_read(op_addr as u16);
            let instr = INSTRUCTION_TABLE[opcode as usize];
            let additional_cycles = self.execute(&instr);
            self.cpu.cycles += additional_cycles + instr.cycles() - 1;
        } else {
            self.cpu.cycles -= 1;
        }
    }

    pub fn reset(&mut self) {
        let lsb = self.bus_read(0xFFFC) as u16;
        let msb = self.bus_read(0xFFFD) as u16;

        self.cpu.reset((msb << 8) | lsb);
        self.clock.reset();
        self.bus.reset();
    }

    fn irq(&mut self) {
        if !self.cpu.status.no_interrupt() {
            self.stack_push((self.cpu.pc >> 8) as u8);
            self.stack_push(self.cpu.pc as u8);
            self.cpu.status.set_b_flag(false);
            self.cpu.status.set_no_interrupt(true);
            self.stack_push(self.cpu.status.0);
            self.cpu.pc = u16::from_le_bytes([self.bus_read(0xFFFE), self.bus_read(0xFFFF)]);
            self.cpu.cycles += 7;
        }
    }

    fn nmi(&mut self) {
        self.stack_push((self.cpu.pc >> 8) as u8);
        self.stack_push(self.cpu.pc as u8);
        self.cpu.status.set_b_flag(false);
        self.cpu.status.set_no_interrupt(true);
        self.stack_push(self.cpu.status.0);
        self.cpu.pc = u16::from_le_bytes([self.bus_read(0xFFFA), self.bus_read(0xFFFB)]);
        self.cpu.cycles += 8;
    }

    fn execute(&mut self, instr: &Instr) -> u8 {
        self.cpu.addr_mode = instr.addr_mode();
        let (addr, page_crossed) = self.addr_mode();
        let page_sensitive = self.operate(instr.op(), addr);
        (page_crossed && page_sensitive) as u8
    }

    fn bus_read(&mut self, addr: u16) -> u8 {
        self.bus.cpu_read(addr).unwrap_or(self.cpu.open_bus)
    }

    fn bus_write(&mut self, addr: u16, data: u8) {
        self.bus.cpu_write(addr, data);
    }

    fn stack_push(&mut self, value: u8) {
        self.bus.cpu_write(0x100 + self.cpu.sp as u16, value);
        self.cpu.sp = self.cpu.sp.wrapping_sub(1);
    }

    fn stack_pull(&mut self) -> u8 {
        self.cpu.sp = self.cpu.sp.wrapping_add(1);
        self.bus_read(0x100 + self.cpu.sp as u16)
    }
}

#[derive(Resource, Default)]
pub struct BreakPointState {
    list: HashSet<u16>,
    new_breakpoint: String,
}

impl BreakPointState {
    pub fn add(&mut self, addr: u16) {
        self.list.insert(addr);
    }

    pub fn remove(&mut self, index: &u16) {
        self.list.remove(index);
    }

    pub fn check(&self, addr: u16) -> bool {
        self.list.contains(&addr)
    }
}

pub struct CpuPlugin;

impl Plugin for CpuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreakPointState>()
            .insert_resource(BreakPointState::default())
            .add_systems(Update, cpu_loop);
    }
}

pub fn cpu_gui(mut query: Query<CpuQuery>, mut contexts: EguiContexts) {
    egui::Window::new("CPU Info").show(&contexts.ctx_mut(), |ui| {
        if let Ok(mut query) = query.get_single_mut() {
            ui.horizontal(|ui| {
                ui.monospace("Status: ");
                ui.monospace(RichText::new("C").color(if query.cpu.status.carry() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(RichText::new("Z").color(if query.cpu.status.zero() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(
                    RichText::new("I").color(if query.cpu.status.no_interrupt() {
                        Color32::GREEN
                    } else {
                        Color32::RED
                    }),
                );
                ui.monospace(RichText::new("D").color(if query.cpu.status.decimal() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(RichText::new("B").color(if query.cpu.status.b_flag() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(RichText::new("1").color(if query.cpu.status.u() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(RichText::new("V").color(if query.cpu.status.overflow() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
                ui.monospace(RichText::new("N").color(if query.cpu.status.negative() {
                    Color32::GREEN
                } else {
                    Color32::RED
                }));
            });
            ui.monospace(format!("A: {a:#04X} ({a:#010b})", a = query.cpu.a));
            ui.monospace(format!("X: {a:#04X} ({a:#010b})", a = query.cpu.x));
            ui.monospace(format!("Y: {a:#04X} ({a:#010b})", a = query.cpu.y));
            ui.monospace(format!("SP: {:#04X}", query.cpu.sp));
            ui.monospace(format!("PC: {:#04X}", query.cpu.pc));
            if ui.button("reset").clicked() {
                query.reset();
            }
            if ui.button("step").clicked() {
                while query.cpu.cycles == 0 {
                    query.clock();
                }
                while query.cpu.cycles != 0 {
                    query.clock();
                }
            }
            if ui.button("next frame").clicked() {
                query.next_frame(None);
            }
            if ui.button("toggle active").clicked() {
                query.clock.enabled = !query.clock.enabled;
            }
        } else {
            ui.label("No CPU found");
        }
    });
}

pub fn disassembly_gui(
    query: Query<CpuQuery>,
    mut breakpoints: ResMut<BreakPointState>,
    mut contexts: EguiContexts,
) {
    egui::Window::new("Disassembly").show(&contexts.ctx_mut(), |ui| {
        if let Ok(query) = query.get_single() {
            let disassembly = query.disassemble(10);
            ScrollArea::vertical().auto_shrink(true).show(ui, |ui| {
                for instr in disassembly {
                    ui.monospace(instr);
                }
            });
        } else {
            ui.label("No CPU found");
        }
        ui.separator();
        ui.label("Breakpoints");
        ui.horizontal(|ui| {
            ui.label("Add breakpoint: ");
            ui.text_edit_singleline(&mut breakpoints.new_breakpoint);
            if ui.button("add").clicked() {
                if let Ok(new_breakpoint) = u16::from_str_radix(
                    breakpoints.new_breakpoint.trim().trim_start_matches("0x"),
                    16,
                ) {
                    breakpoints.add(new_breakpoint);
                } else {
                    println!("Invalid breakpoint");
                }
            }
        });
        egui::Grid::new("breakpoint_grid")
            .num_columns(1)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for breakpoint in breakpoints.list.clone().into_iter() {
                    ui.label(format!("{:#06X}", breakpoint));
                    if ui.button("remove").clicked() {
                        breakpoints.remove(&breakpoint);
                    }
                    ui.end_row();
                }
            });
    });
}

fn cpu_loop(mut query: Query<CpuQuery>, breakpoints: Res<BreakPointState>, time: Res<Time>) {
    if let Ok(mut query) = query.get_single_mut() {
        if query.clock.enabled {
            query.clock.timer.tick(time.delta());
            if query.clock.timer.finished() {
                if query.next_frame(Some(&breakpoints)) {
                    query.clock.enabled = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cartridge::Cartridge, cpu::CpuQuery, nes::NesBundle};
    use bevy::prelude::*;

    macro_rules! setup {
        ($var:ident) => {
            let mut app = App::new();
            let cart = Cartridge::testing();
            app.world_mut().spawn((NesBundle::default(), cart));
            let mut query = app.world_mut().query::<CpuQuery>();
            let mut $var = query.single_mut(app.world_mut());
            $var.cpu.pc = 0x00;
        };
    }
    #[test]
    fn nmi() {
        setup!(query);
        query.cpu.pc = 0x1234;
        query.bus.cpu_write(0xFFFA, 0x21);
        query.bus.cpu_write(0xFFFB, 0x43);

        query.nmi();

        assert_eq!(query.cpu.pc, 0x4321);
        assert_eq!(query.bus.cpu_read(0x01FD), Some(0x12));
        assert_eq!(query.bus.cpu_read(0x01FC), Some(0x34));
        assert_eq!(query.bus.cpu_read(0x01FB), Some(0b0010_0100));
    }
}
