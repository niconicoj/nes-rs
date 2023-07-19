use crate::bus::Bus;

use super::{Cpu, Flag};

impl Cpu {
    pub fn operate(&mut self, opcode: u8) -> bool {
        false
    }

    fn fetch(&self, bus: &Bus) -> u8 {
        self.op_addr.map(|addr| bus.read(addr)).unwrap_or(self.acc)
    }

    fn and(&mut self, bus: &Bus) -> bool {
        let fetched = self.fetch(bus);
        self.acc &= fetched;
        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);
        true
    }

    fn adc(&mut self, bus: &Bus) -> bool {
        let fetched = self.fetch(bus);

        let add_acc = self.acc.overflowing_add(fetched);
        let result = add_acc.0.overflowing_add(self.flags & 0b1);

        self.set_flag(Flag::Carry, add_acc.1 || result.1);
        self.set_flag(Flag::Zero, result.0 == 0);
        self.set_flag(Flag::Negative, result.0 & 0x80 != 0);
        self.set_flag(
            Flag::Overflow,
            (self.acc ^ result.0) & !(self.acc ^ fetched) & 0x80 != 0,
        );

        self.acc = result.0;
        true
    }

    fn sbc(&mut self, bus: &Bus) -> bool {
        let fetched = !self.fetch(bus);

        let add_acc = self.acc.overflowing_add(fetched);
        let result = add_acc.0.overflowing_add(self.flags & 0b1);

        self.set_flag(Flag::Carry, add_acc.1 || result.1);
        self.set_flag(Flag::Zero, result.0 == 0);
        self.set_flag(Flag::Negative, result.0 & 0x80 != 0);
        self.set_flag(
            Flag::Overflow,
            (self.acc ^ result.0) & !(self.acc ^ fetched) & 0x80 != 0,
        );

        self.acc = result.0;
        true
    }

    fn bcc(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Carry));
        true
    }

    fn bcs(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Carry));
        true
    }

    fn beq(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Zero));
        true
    }

    fn bne(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Zero));
        true
    }

    fn branch(&mut self, val: bool) {
        if val {
            self.prg_cntr.0 = self
                .prg_cntr
                .0
                .wrapping_add(self.op_addr.expect("no operand for relative branching"));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bus::Bus,
        cpu::{Cpu, Flag},
    };

    #[test]
    fn and() {
        let mut cpu = Cpu::default();
        cpu.op_addr = Some(0x01);
        cpu.acc = 0b0101_1100;

        let mut bus = Bus::default();
        bus.write(0x01, 0b1101_0110);

        cpu.and(&bus);
        assert_eq!(cpu.acc, 0b01010100);
        assert_eq!(cpu.flags, 0b00000000);

        cpu.op_addr = Some(0x02);

        cpu.and(&bus);
        assert_eq!(cpu.acc, 0b00000000);
        assert_eq!(cpu.flags, 0b00000010);

        cpu.op_addr = Some(0x01);
        cpu.acc = 0b10001100;
        cpu.flags = 0x0;

        cpu.and(&bus);
        assert_eq!(cpu.acc, 0b10000100);
        assert_eq!(cpu.flags, 0b10000000);
    }

    #[test]
    fn adc() {
        let mut cpu = Cpu::default();
        cpu.op_addr = Some(0x01);
        cpu.acc = 34;

        let mut bus = Bus::default();
        bus.write(0x01, 56);

        cpu.adc(&bus);
        assert_eq!(cpu.acc, 90);
        assert_eq!(cpu.flags, 0b00000000);

        cpu.acc = 90;
        bus.write(0x01, 56);
        cpu.adc(&bus);
        assert_eq!(cpu.acc, 146);
        assert_eq!(cpu.flags, 0b11000000);

        bus.write(0x01, 110);
        cpu.flags = 0x0;
        cpu.adc(&bus);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.flags, 0b00000011);

        cpu.flags = 0x0;
        cpu.acc = 135;
        bus.write(0x01, 145);
        cpu.adc(&bus);
        assert_eq!(cpu.acc, 24);
        assert_eq!(cpu.flags, 0b01000001);
    }

    #[test]
    fn sbc() {
        let mut cpu = Cpu::default();
        cpu.op_addr = Some(0x01);
        cpu.acc = 34;

        let mut bus = Bus::default();
        bus.write(0x01, 56);

        cpu.sbc(&bus);
        assert_eq!(cpu.acc, 233);
        assert_eq!(cpu.flags, 0b10000000);

        cpu.flags = 0x0;
        cpu.acc = 90;
        bus.write(0x01, 56);
        cpu.sbc(&bus);
        assert_eq!(cpu.acc, 33);
        assert_eq!(cpu.flags, 0b00000001);

        bus.write(0x01, 110);
        cpu.flags = 0x0;
        cpu.sbc(&bus);
        assert_eq!(cpu.acc, 178);
        assert_eq!(cpu.flags, 0b10000000);

        cpu.flags = 0x0;
        cpu.acc = 135;
        bus.write(0x01, 19);
        cpu.sbc(&bus);
        assert_eq!(cpu.acc, 115);
        assert_eq!(cpu.flags, 0b01000001);
    }

    #[test]
    fn bcc() {
        let mut cpu = Cpu::default();
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Carry, true);
        cpu.bcc();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Carry, false);
        cpu.bcc();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bcc();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn bcs() {
        let mut cpu = Cpu::default();
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Carry, false);
        cpu.bcs();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Carry, true);
        cpu.bcs();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bcs();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn beq() {
        let mut cpu = Cpu::default();
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Zero, false);
        cpu.beq();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Zero, true);
        cpu.beq();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.beq();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn bne() {
        let mut cpu = Cpu::default();
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Zero, true);
        cpu.bne();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Zero, false);
        cpu.bne();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bne();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }
}
