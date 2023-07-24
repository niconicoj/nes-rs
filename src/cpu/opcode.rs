use super::{Cpu, Flag};

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    ADC,AND,ASL,BCC,BCS,BEQ,BIT,BMI,BNE,BPL,BRK,BVC,BVS,CLC,
    CLD,CLI,CLV,CMP,CPX,CPY,DEC,DEX,DEY,EOR,INC,INX,INY,JMP,
    JSR,LDA,LDX,LDY,LSR,NOP,ORA,PHA,PHP,PLA,PLP,ROL,ROR,RTI,
    RTS,SBC,SEC,SED,SEI,STA,STX,STY,TAX,TAY,TSX,TXA,TXS,TYA,
    XXX
}

impl Cpu {
    pub fn operate(&mut self, operation: Operation) -> bool {
        println!("executing operation {:?}", operation);
        match operation {
            Operation::XXX => self.nop(),
            Operation::ADC => self.adc(),
            Operation::AND => self.and(),
            Operation::ASL => self.asl(),
            Operation::BCC => self.bcc(),
            Operation::BCS => self.bcs(),
            Operation::BEQ => self.beq(),
            Operation::BIT => self.bit(),
            Operation::BMI => self.bmi(),
            Operation::BNE => self.bne(),
            Operation::BPL => self.bpl(),
            Operation::BRK => self.brk(),
            Operation::BVC => self.bvc(),
            Operation::BVS => self.bvs(),
            Operation::CLC => self.clc(),
            Operation::CLD => self.cld(),
            Operation::CLI => self.cli(),
            Operation::CLV => self.clv(),
            Operation::CMP => self.cmp(),
            Operation::CPX => self.cpx(),
            Operation::CPY => self.cpy(),
            Operation::DEC => self.dec(),
            Operation::DEX => self.dex(),
            Operation::DEY => self.dey(),
            Operation::EOR => self.eor(),
            Operation::INC => self.inc(),
            Operation::INX => self.inx(),
            Operation::INY => self.iny(),
            Operation::JMP => self.jmp(),
            Operation::JSR => self.jsr(),
            Operation::LDA => self.lda(),
            Operation::LDX => self.ldx(),
            Operation::LDY => self.ldy(),
            Operation::LSR => self.lsr(),
            Operation::NOP => self.nop(),
            Operation::ORA => self.ora(),
            Operation::PHA => self.pha(),
            Operation::PHP => self.php(),
            Operation::PLA => self.pla(),
            Operation::PLP => self.plp(),
            Operation::ROL => self.rol(),
            Operation::ROR => self.ror(),
            Operation::RTI => self.rti(),
            Operation::RTS => self.rts(),
            Operation::SBC => self.sbc(),
            Operation::SEC => self.sec(),
            Operation::SED => self.sed(),
            Operation::SEI => self.sei(),
            Operation::STA => self.sta(),
            Operation::STX => self.stx(),
            Operation::STY => self.sty(),
            Operation::TAX => self.tax(),
            Operation::TAY => self.tay(),
            Operation::TSX => self.tsx(),
            Operation::TXA => self.txa(),
            Operation::TXS => self.txs(),
            Operation::TYA => self.tya(),
        }
    }

    #[inline]
    fn fetch(&self) -> u8 {
        self.op_addr
            .map(|addr| self.bus.read(addr))
            .unwrap_or(self.acc)
    }

    #[inline]
    fn write(&mut self, val: u8) {
        match self.op_addr {
            Some(addr) => self.bus.write(addr, val),
            None => self.acc = val,
        }
    }

    #[inline]
    pub fn and(&mut self) -> bool {
        let fetched = self.fetch();
        self.acc &= fetched;
        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);
        true
    }

    #[inline]
    pub fn adc(&mut self) -> bool {
        let fetched = self.fetch();

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

    #[inline]
    pub fn sbc(&mut self) -> bool {
        let fetched = !self.fetch();

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

    #[inline]
    pub fn bcc(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Carry));
        true
    }

    #[inline]
    pub fn bcs(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Carry));
        true
    }

    #[inline]
    pub fn beq(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Zero));
        true
    }

    #[inline]
    pub fn bne(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Zero));
        true
    }

    #[inline]
    pub fn bmi(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Negative));
        true
    }

    #[inline]
    pub fn bpl(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Negative));
        true
    }

    #[inline]
    pub fn bvc(&mut self) -> bool {
        self.branch(!self.get_flag(Flag::Overflow));
        true
    }

    #[inline]
    pub fn bvs(&mut self) -> bool {
        self.branch(self.get_flag(Flag::Overflow));
        true
    }

    #[inline]
    fn branch(&mut self, val: bool) {
        if val {
            self.prg_cntr.0 = self
                .prg_cntr
                .0
                .wrapping_add(self.op_addr.expect("no operand for relative branching"));
        }
    }

    #[inline]
    pub fn brk(&mut self) -> bool {
        self.set_flag(Flag::NoInterupts, true);
        let prg_cntr_bytes = self.prg_cntr.0.to_be_bytes();
        self.stk_push(prg_cntr_bytes[0]);
        self.stk_push(prg_cntr_bytes[1]);

        self.set_flag(Flag::Break, true);
        self.stk_push(self.flags);
        self.set_flag(Flag::Break, false);

        self.prg_cntr.0 = (self.bus.read(0xFFFE) as u16) | (self.bus.read(0xFFFF) as u16) << 8;
        false
    }

    #[inline]
    pub fn clc(&mut self) -> bool {
        self.set_flag(Flag::Carry, false);
        false
    }

    #[inline]
    pub fn cld(&mut self) -> bool {
        self.set_flag(Flag::DecimalMode, false);
        false
    }

    #[inline]
    pub fn cli(&mut self) -> bool {
        self.set_flag(Flag::NoInterupts, false);
        false
    }

    #[inline]
    pub fn clv(&mut self) -> bool {
        self.set_flag(Flag::Overflow, false);
        false
    }

    #[inline]
    pub fn asl(&mut self) -> bool {
        let fetched = self.fetch();
        let result = fetched << 1;

        self.set_flag(Flag::Carry, result < fetched);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, result & 0x80 != 0);

        self.write(result);

        false
    }

    #[inline]
    pub fn lsr(&mut self) -> bool {
        let fetched = self.fetch();
        let result = fetched >> 1;

        self.set_flag(Flag::Carry, result > fetched);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, false);

        self.write(result);

        false
    }

    #[inline]
    pub fn rol(&mut self) -> bool {
        let fetched = self.fetch();
        let result = fetched << 1 | self.get_flag(Flag::Carry) as u8;

        self.set_flag(Flag::Carry, fetched & 0x80 != 0);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, result & 0x80 != 0);

        self.write(result);

        false
    }

    #[inline]
    pub fn ror(&mut self) -> bool {
        let fetched = self.fetch();
        let result = fetched >> 1 | (self.get_flag(Flag::Carry) as u8) << 7;

        self.set_flag(Flag::Carry, fetched & 0x1 != 0);
        self.set_flag(Flag::Zero, result == 0);
        self.set_flag(Flag::Negative, result & 0x80 != 0);

        self.write(result);

        false
    }

    #[inline]
    pub fn bit(&mut self) -> bool {
        let fetched = self.fetch();

        self.set_flag(Flag::Zero, fetched & self.acc == 0);
        self.set_flag(Flag::Overflow, fetched & 0x40 != 0);
        self.set_flag(Flag::Negative, fetched & 0x80 != 0);

        false
    }

    #[inline]
    pub fn cmp(&mut self) -> bool {
        self.compare(self.acc);
        true
    }

    #[inline]
    pub fn cpx(&mut self) -> bool {
        self.compare(self.x);
        true
    }

    #[inline]
    pub fn cpy(&mut self) -> bool {
        self.compare(self.y);
        true
    }

    #[inline]
    fn compare(&mut self, val: u8) {
        let fetched = self.fetch();
        self.set_flag(Flag::Carry, val >= fetched);
        self.set_flag(Flag::Zero, val == fetched);
        self.set_flag(Flag::Negative, val.wrapping_sub(fetched) & 0x80 != 0);
    }

    #[inline]
    pub fn dec(&mut self) -> bool {
        let val = self.fetch();
        let result = self.decrement(val);
        self.write(result);
        false
    }

    #[inline]
    pub fn dex(&mut self) -> bool {
        self.x = self.decrement(self.x);
        false
    }

    #[inline]
    pub fn dey(&mut self) -> bool {
        self.y = self.decrement(self.y);
        false
    }

    #[inline]
    fn decrement(&mut self, val: u8) -> u8 {
        let val = val.wrapping_sub(1);
        self.set_flag(Flag::Zero, val == 0);
        self.set_flag(Flag::Negative, val & 0x80 != 0);
        val
    }

    #[inline]
    pub fn eor(&mut self) -> bool {
        let fetched = self.fetch();
        self.acc = fetched ^ self.acc;

        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);

        true
    }

    #[inline]
    pub fn inc(&mut self) -> bool {
        let val = self.fetch();
        let result = self.increment(val);
        self.write(result);
        false
    }

    #[inline]
    pub fn inx(&mut self) -> bool {
        self.x = self.increment(self.x);
        false
    }

    #[inline]
    pub fn iny(&mut self) -> bool {
        self.y = self.increment(self.y);
        false
    }

    fn increment(&mut self, val: u8) -> u8 {
        let val = val.wrapping_add(1);
        self.set_flag(Flag::Zero, val == 0);
        self.set_flag(Flag::Negative, val & 0x80 != 0);
        val
    }

    #[inline]
    pub fn jmp(&mut self) -> bool {
        self.prg_cntr.0 = self.op_addr.expect("no operand for jump");
        false
    }

    #[inline]
    pub fn jsr(&mut self) -> bool {
        let prg_cntr_bytes = self.prg_cntr.0.to_be_bytes();
        self.stk_push(prg_cntr_bytes[0]);
        self.stk_push(prg_cntr_bytes[1]);

        self.prg_cntr.0 = self.op_addr.expect("no operand for jsr");

        false
    }

    #[inline]
    pub fn lda(&mut self) -> bool {
        self.acc = self.fetch();

        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);

        true
    }

    #[inline]
    pub fn ldx(&mut self) -> bool {
        self.x = self.fetch();

        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 != 0);

        true
    }

    #[inline]
    pub fn ldy(&mut self) -> bool {
        self.y = self.fetch();

        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 != 0);

        true
    }

    #[inline]
    pub fn nop(&self) -> bool {
        false
    }

    #[inline]
    pub fn ora(&mut self) -> bool {
        self.acc = self.acc | self.fetch();

        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);

        true
    }

    #[inline]
    pub fn pha(&mut self) -> bool {
        self.stk_push(self.acc);
        false
    }

    #[inline]
    pub fn php(&mut self) -> bool {
        self.stk_push(self.flags);
        false
    }

    #[inline]
    pub fn pla(&mut self) -> bool {
        self.acc = self.stk_pull();
        false
    }

    #[inline]
    pub fn plp(&mut self) -> bool {
        self.flags = self.stk_pull();
        false
    }

    #[inline]
    pub fn rti(&mut self) -> bool {
        self.flags = self.stk_pull();
        self.set_flag(Flag::Break, false);
        self.set_flag(Flag::NoInterupts, false);

        self.prg_cntr.0 = u16::from_le_bytes([self.stk_pull(), self.stk_pull()]);
        false
    }

    #[inline]
    pub fn rts(&mut self) -> bool {
        self.prg_cntr.0 = u16::from_le_bytes([self.stk_pull(), self.stk_pull()]);
        self.prg_cntr.0 += 1;
        false
    }

    #[inline]
    pub fn sec(&mut self) -> bool {
        self.set_flag(Flag::Carry, true);
        false
    }

    #[inline]
    pub fn sed(&mut self) -> bool {
        self.set_flag(Flag::DecimalMode, true);
        false
    }

    #[inline]
    pub fn sei(&mut self) -> bool {
        self.set_flag(Flag::NoInterupts, true);
        false
    }

    #[inline]
    pub fn sta(&mut self) -> bool {
        self.write(self.acc);
        false
    }

    #[inline]
    pub fn stx(&mut self) -> bool {
        self.write(self.x);
        false
    }

    #[inline]
    pub fn sty(&mut self) -> bool {
        self.write(self.y);
        false
    }

    #[inline]
    pub fn tax(&mut self) -> bool {
        self.x = self.acc;

        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 != 0);

        false
    }

    #[inline]
    pub fn tay(&mut self) -> bool {
        self.y = self.acc;

        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 != 0);

        false
    }

    #[inline]
    pub fn tsx(&mut self) -> bool {
        self.x = self.stk_ptr;

        self.set_flag(Flag::Zero, self.x == 0);
        self.set_flag(Flag::Negative, self.x & 0x80 != 0);

        false
    }

    #[inline]
    pub fn txa(&mut self) -> bool {
        self.acc = self.x;

        self.set_flag(Flag::Zero, self.acc == 0);
        self.set_flag(Flag::Negative, self.acc & 0x80 != 0);

        false
    }

    #[inline]
    pub fn txs(&mut self) -> bool {
        self.stk_ptr = self.x;

        self.set_flag(Flag::Zero, self.stk_ptr == 0);
        self.set_flag(Flag::Negative, self.stk_ptr & 0x80 != 0);

        false
    }

    #[inline]
    pub fn tya(&mut self) -> bool {
        self.acc = self.y;

        self.set_flag(Flag::Zero, self.y == 0);
        self.set_flag(Flag::Negative, self.y & 0x80 != 0);

        false
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
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.op_addr = Some(0x01);
        cpu.acc = 0b0101_1100;

        cpu.bus.write(0x01, 0b1101_0110);

        cpu.and();
        assert_eq!(cpu.acc, 0b01010100);
        assert_eq!(cpu.flags, 0b00000000);

        cpu.op_addr = Some(0x02);

        cpu.and();
        assert_eq!(cpu.acc, 0b00000000);
        assert_eq!(cpu.flags, 0b00000010);

        cpu.op_addr = Some(0x01);
        cpu.acc = 0b10001100;
        cpu.flags = 0x0;

        cpu.and();
        assert_eq!(cpu.acc, 0b10000100);
        assert_eq!(cpu.flags, 0b10000000);
    }

    #[test]
    fn adc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.op_addr = Some(0x01);
        cpu.acc = 34;
        cpu.bus.write(0x01, 56);

        cpu.adc();
        assert_eq!(cpu.acc, 90);
        assert_eq!(cpu.flags, 0b00000000);

        cpu.acc = 90;
        cpu.bus.write(0x01, 56);
        cpu.adc();
        assert_eq!(cpu.acc, 146);
        assert_eq!(cpu.flags, 0b11000000);

        cpu.bus.write(0x01, 110);
        cpu.flags = 0x0;
        cpu.adc();
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.flags, 0b00000011);

        cpu.flags = 0x0;
        cpu.acc = 135;
        cpu.bus.write(0x01, 145);
        cpu.adc();
        assert_eq!(cpu.acc, 24);
        assert_eq!(cpu.flags, 0b01000001);
    }

    #[test]
    fn sbc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.op_addr = Some(0x01);
        cpu.acc = 34;

        cpu.bus.write(0x01, 56);

        cpu.sbc();
        assert_eq!(cpu.acc, 233);
        assert_eq!(cpu.flags, 0b10000000);

        cpu.flags = 0x0;
        cpu.acc = 90;
        cpu.bus.write(0x01, 56);
        cpu.sbc();
        assert_eq!(cpu.acc, 33);
        assert_eq!(cpu.flags, 0b00000001);

        cpu.bus.write(0x01, 110);
        cpu.flags = 0x0;
        cpu.sbc();
        assert_eq!(cpu.acc, 178);
        assert_eq!(cpu.flags, 0b10000000);

        cpu.flags = 0x0;
        cpu.acc = 135;
        cpu.bus.write(0x01, 19);
        cpu.sbc();
        assert_eq!(cpu.acc, 115);
        assert_eq!(cpu.flags, 0b01000001);
    }

    #[test]
    fn bcc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
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
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
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
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
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
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
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

    #[test]
    fn bmi() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Negative, false);
        cpu.bmi();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Negative, true);
        cpu.bmi();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bmi();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn bpl() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Negative, true);
        cpu.bpl();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Negative, false);
        cpu.bpl();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bpl();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn bvc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Overflow, true);
        cpu.bvc();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Overflow, false);
        cpu.bvc();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bvc();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn bvs() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        let addr = 155;
        cpu.prg_cntr.0 = 0;
        cpu.op_addr = Some((addr as u16) | 0xFF00);

        cpu.set_flag(Flag::Overflow, false);
        cpu.bvs();
        assert_eq!(cpu.prg_cntr.0, 0);

        cpu.set_flag(Flag::Overflow, true);
        cpu.bvs();
        assert_eq!(cpu.prg_cntr.0, 65435);

        cpu.op_addr = Some(13);
        cpu.bvs();
        assert_eq!(cpu.prg_cntr.0, 65448);
    }

    #[test]
    fn asl() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.prg_cntr.0 = 0;
        cpu.acc = 123;
        cpu.op_addr = Some(0x0001);

        cpu.bus.write(0x01, 56);
        cpu.bus.write(0x02, 156);
        cpu.bus.write(0x03, 128);

        cpu.op_addr = None;
        test_asl(&mut cpu, 246, 0x80);
        cpu.op_addr = Some(0x00);
        test_asl(&mut cpu, 0, 0x02);
        cpu.op_addr = Some(0x01);
        test_asl(&mut cpu, 112, 0x00);
        cpu.op_addr = Some(0x02);
        test_asl(&mut cpu, 56, 0x01);
        cpu.op_addr = Some(0x03);
        test_asl(&mut cpu, 0, 0x03);
    }

    fn test_asl(cpu: &mut Cpu, expect: u8, flags: u8) {
        assert!(!cpu.asl());
        match cpu.op_addr {
            Some(addr) => assert_eq!(cpu.bus.read(addr), expect),
            None => assert_eq!(cpu.acc, expect),
        }
        assert_eq!(
            cpu.flags, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, cpu.flags
        );
    }

    #[test]
    fn lsr() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.prg_cntr.0 = 0;
        cpu.acc = 123;
        cpu.op_addr = Some(0x0001);

        cpu.bus.write(0x01, 56);
        cpu.bus.write(0x02, 156);
        cpu.bus.write(0x03, 1);

        cpu.op_addr = None;
        test_lsr(&mut cpu, 61, 0x00);
        cpu.op_addr = Some(0x00);
        test_lsr(&mut cpu, 0, 0x02);
        cpu.op_addr = Some(0x01);
        test_lsr(&mut cpu, 28, 0x00);
        cpu.op_addr = Some(0x02);
        test_lsr(&mut cpu, 78, 0x00);
        cpu.op_addr = Some(0x03);
        test_lsr(&mut cpu, 0, 0x02);
    }

    fn test_lsr(cpu: &mut Cpu, expect: u8, flags: u8) {
        assert!(!cpu.lsr());
        match cpu.op_addr {
            Some(addr) => assert_eq!(cpu.bus.read(addr), expect),
            None => assert_eq!(cpu.acc, expect),
        }
        assert_eq!(
            cpu.flags, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, cpu.flags
        );
    }

    #[test]
    fn rol() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.prg_cntr.0 = 0;
        cpu.acc = 123;
        cpu.op_addr = Some(0x0001);

        cpu.bus.write(0x01, 56);
        cpu.bus.write(0x02, 156);
        cpu.bus.write(0x03, 255);
        cpu.bus.write(0x04, 1);

        cpu.op_addr = None;
        test_rol(&mut cpu, 246, 0x80);
        cpu.op_addr = Some(0x00);
        test_rol(&mut cpu, 0, 0x02);
        cpu.op_addr = Some(0x01);
        test_rol(&mut cpu, 112, 0x00);
        cpu.op_addr = Some(0x02);
        test_rol(&mut cpu, 56, 0x01);
        cpu.op_addr = Some(0x03);
        test_rol(&mut cpu, 255, 0x81);
        cpu.op_addr = Some(0x04);
        test_rol(&mut cpu, 3, 0x00);
    }

    fn test_rol(cpu: &mut Cpu, expect: u8, flags: u8) {
        assert!(!cpu.rol());
        match cpu.op_addr {
            Some(addr) => assert_eq!(
                cpu.bus.read(addr),
                expect,
                "invalid addr {} => expected {} but was {}",
                addr,
                expect,
                cpu.bus.read(addr)
            ),
            None => assert_eq!(
                cpu.acc, expect,
                "invalid accumulator => expected {} but was {}",
                expect, cpu.acc
            ),
        }
        assert_eq!(
            cpu.flags, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, cpu.flags
        );
    }

    #[test]
    fn ror() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.prg_cntr.0 = 0;
        cpu.acc = 123;
        cpu.op_addr = Some(0x0001);

        cpu.bus.write(0x01, 56);
        cpu.bus.write(0x02, 156);
        cpu.bus.write(0x03, 255);
        cpu.bus.write(0x04, 37);

        cpu.op_addr = None;
        test_ror(&mut cpu, 61, 0x01);
        cpu.op_addr = Some(0x00);
        test_ror(&mut cpu, 128, 0x80);
        cpu.op_addr = Some(0x01);
        test_ror(&mut cpu, 28, 0x00);
        cpu.op_addr = Some(0x02);
        test_ror(&mut cpu, 78, 0x00);
        cpu.op_addr = Some(0x03);
        test_ror(&mut cpu, 127, 0x01);
        cpu.op_addr = Some(0x04);
        test_ror(&mut cpu, 146, 0x81);
    }

    fn test_ror(cpu: &mut Cpu, expect: u8, flags: u8) {
        assert!(!cpu.ror());
        match cpu.op_addr {
            Some(addr) => assert_eq!(
                cpu.bus.read(addr),
                expect,
                "invalid addr {} => expected {} but was {}",
                addr,
                expect,
                cpu.bus.read(addr)
            ),
            None => assert_eq!(
                cpu.acc, expect,
                "invalid accumulator => expected {} but was {}",
                expect, cpu.acc
            ),
        }
        assert_eq!(
            cpu.flags, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, cpu.flags
        );
    }

    #[test]
    fn bit() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.prg_cntr.0 = 0;
        cpu.acc = 0b01101001;

        cpu.op_addr = Some(0x00);
        cpu.bus.write(0x00, 0b10010110);
        test_bit(&mut cpu, 0x82);

        cpu.op_addr = Some(0x01);
        cpu.bus.write(0x01, 0b00110111);
        test_bit(&mut cpu, 0x00);

        cpu.op_addr = Some(0x02);
        cpu.bus.write(0x02, 0b11010000);
        test_bit(&mut cpu, 0xC0);
    }

    fn test_bit(cpu: &mut Cpu, flags: u8) {
        assert!(!cpu.bit());
        assert_eq!(
            cpu.flags, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, cpu.flags
        );
    }

    #[test]
    fn brk() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.flags = 0x82;
        cpu.prg_cntr.0 = 0x1234;
        cpu.bus.write(0xFFFE, 0x56);
        cpu.bus.write(0xFFFF, 0x78);

        assert!(!cpu.brk());
        assert_eq!(cpu.bus.read(0x01FF), 0x12);
        assert_eq!(cpu.bus.read(0x01FE), 0x34);
        assert_eq!(cpu.bus.read(0x01FD), 0x82 | 0x04 | 0x10);
        assert_eq!(cpu.prg_cntr.0, 0x7856);
        assert_eq!(
            cpu.flags,
            0x82 | 0x04,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            0x40,
            cpu.flags
        );
    }

    #[test]
    fn clc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.flags = 0xFF;
        cpu.clc();
        assert_eq!(cpu.flags, 0xFE);
    }

    #[test]
    fn cld() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.flags = 0xFF;
        cpu.cld();
        assert_eq!(cpu.flags, 0xF7);
    }

    #[test]
    fn cli() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.flags = 0xFF;
        cpu.cli();
        assert_eq!(cpu.flags, 0xFB);
    }

    #[test]
    fn clv() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.flags = 0xFF;
        cpu.clv();
        assert_eq!(cpu.flags, 0xBF);
    }

    #[test]
    fn cmp() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.acc = 0x56;
        cpu.op_addr = Some(0x12);
        cpu.bus.write(0x12, 0x34);
        assert!(cpu.cmp());
        assert_eq!(cpu.flags, 0x01);

        cpu.acc = 0x01;
        cpu.op_addr = Some(0x13);
        cpu.bus.write(0x13, 0x34);
        assert!(cpu.cmp());
        assert_eq!(cpu.flags, 0x80);

        cpu.acc = 0x01;
        cpu.op_addr = Some(0x14);
        cpu.bus.write(0x14, 0x01);
        assert!(cpu.cmp());
        assert_eq!(cpu.flags, 0x03);
    }

    #[test]
    fn cpx() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.x = 0x56;
        cpu.op_addr = Some(0x12);
        cpu.bus.write(0x12, 0x34);
        assert!(cpu.cpx());
        assert_eq!(cpu.flags, 0x01);

        cpu.x = 0x01;
        cpu.op_addr = Some(0x13);
        cpu.bus.write(0x13, 0x34);
        assert!(cpu.cpx());
        assert_eq!(cpu.flags, 0x80);

        cpu.x = 0x01;
        cpu.op_addr = Some(0x14);
        cpu.bus.write(0x14, 0x01);
        assert!(cpu.cpx());
        assert_eq!(cpu.flags, 0x03);
    }

    #[test]
    fn cpy() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.y = 0x56;
        cpu.op_addr = Some(0x12);
        cpu.bus.write(0x12, 0x34);
        assert!(cpu.cpy());
        assert_eq!(cpu.flags, 0x01);

        cpu.y = 0x01;
        cpu.op_addr = Some(0x13);
        cpu.bus.write(0x13, 0x34);
        assert!(cpu.cpy());
        assert_eq!(cpu.flags, 0x80);

        cpu.y = 0x01;
        cpu.op_addr = Some(0x14);
        cpu.bus.write(0x14, 0x01);
        assert!(cpu.cpy());
        assert_eq!(cpu.flags, 0x03);
    }

    #[test]
    fn dec() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x12);
        cpu.bus.write(0x12, 0x34);
        assert!(!cpu.dec());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.bus.read(0x12), 0x33);

        cpu.op_addr = Some(0x13);
        cpu.bus.write(0x13, 0x84);
        assert!(!cpu.dec());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.bus.read(0x13), 0x83);

        cpu.op_addr = Some(0x14);
        cpu.bus.write(0x14, 0x01);
        assert!(!cpu.dec());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.bus.read(0x14), 0x00);
    }

    #[test]
    fn dex() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.x = 0x34;
        assert!(!cpu.dex());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.x, 0x33);

        cpu.x = 0x84;
        assert!(!cpu.dex());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.x, 0x83);

        cpu.x = 0x01;
        assert!(!cpu.dex());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.x, 0x00);
    }

    #[test]
    fn dey() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.y = 0x34;
        assert!(!cpu.dey());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.y, 0x33);

        cpu.y = 0x84;
        assert!(!cpu.dey());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.y, 0x83);

        cpu.y = 0x01;
        assert!(!cpu.dey());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.y, 0x00);
    }

    #[test]
    fn eor() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x12);
        cpu.acc = 0b11101110;
        cpu.bus.write(0x12, 0b01101010);
        assert!(cpu.eor());
        assert_eq!(cpu.acc, 0b10000100);
        assert_eq!(cpu.flags, 0x80);

        cpu.op_addr = Some(0x12);
        cpu.acc = 0b11110011;
        cpu.bus.write(0x12, 0b11110011);
        assert!(cpu.eor());
        assert_eq!(cpu.acc, 0b0);
        assert_eq!(cpu.flags, 0x02);

        cpu.op_addr = Some(0x12);
        cpu.acc = 0b11110000;
        cpu.bus.write(0x12, 0b11110011);
        assert!(cpu.eor());
        assert_eq!(cpu.acc, 0b11);
        assert_eq!(cpu.flags, 0x00);
    }

    #[test]
    fn inc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x12);
        cpu.bus.write(0x12, 0x34);
        assert!(!cpu.inc());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.bus.read(0x12), 0x35);

        cpu.op_addr = Some(0x13);
        cpu.bus.write(0x13, 0x84);
        assert!(!cpu.inc());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.bus.read(0x13), 0x85);

        cpu.op_addr = Some(0x14);
        cpu.bus.write(0x14, 0xFF);
        assert!(!cpu.inc());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.bus.read(0x14), 0x00);
    }

    #[test]
    fn inx() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.x = 0x34;
        assert!(!cpu.inx());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.x, 0x35);

        cpu.x = 0x84;
        assert!(!cpu.inx());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.x, 0x85);

        cpu.x = 0xFF;
        assert!(!cpu.inx());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.x, 0x00);
    }

    #[test]
    fn iny() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.y = 0x34;
        assert!(!cpu.iny());
        assert_eq!(cpu.flags, 0x00);
        assert_eq!(cpu.y, 0x35);

        cpu.y = 0x84;
        assert!(!cpu.iny());
        assert_eq!(cpu.flags, 0x80);
        assert_eq!(cpu.y, 0x85);

        cpu.y = 0xFF;
        assert!(!cpu.iny());
        assert_eq!(cpu.flags, 0x02);
        assert_eq!(cpu.y, 0x00);
    }

    #[test]
    fn jmp() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x1234);
        assert!(!cpu.jmp());
        assert_eq!(cpu.prg_cntr.0, 0x1234);
    }

    #[test]
    fn jsr() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x1234);
        cpu.prg_cntr.0 = 0x5678;
        cpu.stk_ptr = 0xFF;

        assert!(!cpu.jsr());

        assert_eq!(cpu.bus.read(0x01FF), 0x56);
        assert_eq!(cpu.bus.read(0x01FE), 0x78);
        assert_eq!(cpu.prg_cntr.0, 0x1234);
    }

    #[test]
    fn lda() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x1234);
        cpu.bus.write(0x1234, 0x56);

        assert!(cpu.lda());
        assert_eq!(cpu.acc, 0x56);
    }

    #[test]
    fn ldx() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x1234);
        cpu.bus.write(0x1234, 0x56);

        assert!(cpu.ldx());
        assert_eq!(cpu.x, 0x56);
    }

    #[test]
    fn ldy() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.op_addr = Some(0x1234);
        cpu.bus.write(0x1234, 0x56);

        assert!(cpu.ldy());
        assert_eq!(cpu.y, 0x56);
    }

    #[test]
    fn pha() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.acc = 0x12;
        assert!(!cpu.pha());
        assert_eq!(cpu.bus.read(0x01FF), 0x12);

        cpu.acc = 0x34;
        assert!(!cpu.pha());
        assert_eq!(cpu.bus.read(0x01FE), 0x34);
    }

    #[test]
    fn php() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.flags = 0x12;
        assert!(!cpu.php());
        assert_eq!(cpu.bus.read(0x01FF), 0x12);

        cpu.flags = 0x34;
        assert!(!cpu.php());
        assert_eq!(cpu.bus.read(0x01FE), 0x34);
    }

    #[test]
    fn pla() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.stk_push(0x12);
        cpu.stk_push(0x23);

        assert!(!cpu.pla());
        assert_eq!(cpu.acc, 0x23);
        assert!(!cpu.pla());
        assert_eq!(cpu.acc, 0x12);
    }

    #[test]
    fn plp() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.stk_push(0x12);
        cpu.stk_push(0x23);

        assert!(!cpu.plp());
        assert_eq!(cpu.flags, 0x23);
        assert!(!cpu.plp());
        assert_eq!(cpu.flags, 0x12);
    }

    #[test]
    fn rti() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.stk_push(0x12);
        cpu.stk_push(0x34);
        cpu.stk_push(0x56);

        assert!(!cpu.rti());
        assert_eq!(cpu.flags, 0x56 & 0xEF & 0xFB);
        assert_eq!(cpu.prg_cntr.0, 0x1234);
    }

    #[test]
    fn rts() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.bus.write(0x01FF, 0x34);
        cpu.bus.write(0x01FE, 0x65);
        cpu.prg_cntr.0 = 0x1234;
        cpu.stk_ptr = 0xFD;

        assert!(!cpu.rts());
        assert_eq!(cpu.prg_cntr.0, 0x3466);
    }

    #[test]
    fn sec() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.get_flag(Flag::Carry));
        assert!(!cpu.sec());
        assert!(cpu.get_flag(Flag::Carry));
    }

    #[test]
    fn sed() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.get_flag(Flag::DecimalMode));
        assert!(!cpu.sed());
        assert!(cpu.get_flag(Flag::DecimalMode));
    }

    #[test]
    fn sei() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.get_flag(Flag::NoInterupts));
        assert!(!cpu.sei());
        assert!(cpu.get_flag(Flag::NoInterupts));
    }

    #[test]
    fn sta() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.acc = 0x12;
        cpu.op_addr = Some(0x3456);

        assert!(!cpu.sta());
        assert_eq!(cpu.bus.read(0x3456), 0x12);
    }

    #[test]
    fn stx() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.x = 0x12;
        cpu.op_addr = Some(0x3456);

        assert!(!cpu.stx());
        assert_eq!(cpu.bus.read(0x3456), 0x12);
    }

    #[test]
    fn sty() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.y = 0x12;
        cpu.op_addr = Some(0x3456);

        assert!(!cpu.sty());
        assert_eq!(cpu.bus.read(0x3456), 0x12);
    }

    #[test]
    fn tax() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.tax());
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.acc = 0x23;
        assert!(!cpu.tax());
        assert_eq!(cpu.x, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.acc = 0xB2;
        assert!(!cpu.tax());
        assert_eq!(cpu.x, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }

    #[test]
    fn tay() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.tay());
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.acc = 0x23;
        assert!(!cpu.tay());
        assert_eq!(cpu.y, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.acc = 0xB2;
        assert!(!cpu.tay());
        assert_eq!(cpu.y, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }

    #[test]
    fn tsx() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        cpu.stk_ptr = 0x00;
        assert!(!cpu.tsx());
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.stk_ptr = 0x23;
        assert!(!cpu.tsx());
        assert_eq!(cpu.x, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.stk_ptr = 0xB2;
        assert!(!cpu.tsx());
        assert_eq!(cpu.x, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }

    #[test]
    fn txa() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.txa());
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.x = 0x23;
        assert!(!cpu.txa());
        assert_eq!(cpu.acc, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.x = 0xB2;
        assert!(!cpu.txa());
        assert_eq!(cpu.acc, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }

    #[test]
    fn txs() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.txs());
        assert_eq!(cpu.stk_ptr, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.x = 0x23;
        assert!(!cpu.txs());
        assert_eq!(cpu.stk_ptr, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.x = 0xB2;
        assert!(!cpu.txs());
        assert_eq!(cpu.stk_ptr, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }

    #[test]
    fn tya() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);

        assert!(!cpu.tya());
        assert_eq!(cpu.acc, 0x00);
        assert_eq!(cpu.flags, 0x02);

        cpu.y = 0x23;
        assert!(!cpu.tya());
        assert_eq!(cpu.acc, 0x23);
        assert_eq!(cpu.flags, 0x00);

        cpu.y = 0xB2;
        assert!(!cpu.tya());
        assert_eq!(cpu.acc, 0xB2);
        assert_eq!(cpu.flags, 0x80);
    }
}
