use crate::{bus::CpuBus, cartridge::Cartridge};

use self::instr::{Instr, INSTRUCTION_TABLE};

mod addr_mode;
mod instr;
mod opcode;

enum Flag {
    Carry = 0x01,
    Zero = 0x02,
    NoInterupts = 0x04,
    DecimalMode = 0x08,
    Break = 0x10,
    Overflow = 0x40,
    Negative = 0x80,
}

#[derive(Default)]
struct ProgramCounter(u16);
impl ProgramCounter {
    pub fn adv(&mut self) -> u16 {
        let val = self.0;
        self.0 = self.0.wrapping_add(1);
        val
    }
}

#[derive(Default)]
pub struct Cpu {
    bus: CpuBus,
    acc: u8,
    x: u8,
    y: u8,
    stk_ptr: u8,
    prg_cntr: ProgramCounter,
    flags: u8,
    cycles: usize,

    op_addr: Option<u16>,
}

impl Cpu {
    pub fn new(bus: CpuBus) -> Self {
        Self {
            bus,
            acc: 0,
            x: 0,
            y: 0,
            stk_ptr: 0xFF,
            prg_cntr: ProgramCounter(0),
            flags: 0,
            cycles: 0,
            op_addr: None,
        }
    }

    pub fn plug_cartridge(&mut self, cartridge: &Cartridge) {
        self.bus.plug_cartridge(cartridge);
    }

    pub fn unplug_cartridge(&mut self) {
        self.bus.unplug_cartridge();
    }

    pub fn tick(&mut self) {
        if self.cycles == 0 {
            let opcode = self.bus.read(self.prg_cntr.adv());
            let instruction = INSTRUCTION_TABLE[opcode as usize];
            let add_cycle = self.exec(&instruction);
            self.cycles += (add_cycle as usize) + instruction.cycles();
        }
        self.cycles -= 1;
    }

    fn exec(&mut self, instruction: &Instr) -> bool {
        println!("exec instr : {:?}", instruction);
        let addr_result = self.addr_mode(instruction.addr_mode());
        let op_result = self.operate(instruction.op());
        addr_result && op_result
    }

    pub fn reset(&mut self) {}
    // standard interrupt request
    pub fn irq(&mut self) {}
    // non-maskable interrupt request
    pub fn nmi(&mut self) {}

    fn set_flag(&mut self, flag: Flag, value: bool) {
        self.flags = match value {
            true => self.flags | flag as u8,
            false => self.flags & !(flag as u8),
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        self.flags & (flag as u8) != 0
    }

    pub fn stk_push(&mut self, data: u8) {
        self.bus
            .write(0x100_u16.wrapping_add(self.stk_ptr as u16), data);
        self.stk_ptr = self.stk_ptr.wrapping_sub(1);
    }

    pub fn stk_pull(&mut self) -> u8 {
        self.stk_ptr = self.stk_ptr.wrapping_add(1);
        self.bus.read(0x100_u16.wrapping_add(self.stk_ptr as u16))
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::CpuBus;

    use super::{Cpu, Flag};

    #[test]
    fn set_flag() {
        let bus = CpuBus::default();
        let mut cpu = Cpu::new(bus);

        assert_eq!(cpu.flags, 0b00000000);
        cpu.set_flag(Flag::Zero, true);
        assert_eq!(cpu.flags, 0b00000010);
        cpu.set_flag(Flag::Break, true);
        assert_eq!(cpu.flags, 0b00010010);
        cpu.set_flag(Flag::Negative, true);
        assert_eq!(cpu.flags, 0b10010010);
        cpu.set_flag(Flag::Break, true);
        assert_eq!(cpu.flags, 0b10010010);
        cpu.set_flag(Flag::Break, false);
        assert_eq!(cpu.flags, 0b10000010);
        cpu.set_flag(Flag::Carry, true);
        assert_eq!(cpu.flags, 0b10000011);
    }

    #[test]
    fn get_flag() {
        let bus = CpuBus::default();
        let mut cpu = Cpu::new(bus);

        cpu.flags = 0b00110110;

        assert!(cpu.get_flag(Flag::Zero));
        assert!(!cpu.get_flag(Flag::Carry));
        assert!(!cpu.get_flag(Flag::Negative));

        cpu.flags = 0b00110101;
        assert!(cpu.get_flag(Flag::Carry));
    }

    #[test]
    fn tick() {
        let mut bus = CpuBus::default();
        // LDA 0x12
        bus.write(0x0000, 0xA9);
        bus.write(0x0001, 0x12);
        //ADC 0x34
        bus.write(0x0002, 0x69);
        bus.write(0x0003, 0x34);

        let mut cpu = Cpu::new(bus);
        cpu.tick();
        assert_eq!(cpu.acc, 0x12);
        assert_eq!(cpu.cycles, 1);
        cpu.tick();
        assert_eq!(cpu.acc, 0x12);
        assert_eq!(cpu.cycles, 0);
        cpu.tick();
        assert_eq!(cpu.acc, 0x46);
        assert_eq!(cpu.cycles, 1);
    }
}
