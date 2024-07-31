use super::CpuQueryItem;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
pub enum Op {
    ADC,AND,ASL,BCC,BCS,BEQ,BIT,BMI,BNE,BPL,BRK,BVC,BVS,CLC,
    CLD,CLI,CLV,CMP,CPX,CPY,DEC,DEX,DEY,EOR,INC,INX,INY,JMP,
    JSR,LDA,LDX,LDY,LSR,NOP,ORA,PHA,PHP,PLA,PLP,ROL,ROR,RTI,
    RTS,SBC,SEC,SED,SEI,STA,STX,STY,TAX,TAY,TSX,TXA,TXS,TYA,
    XXX
}

impl<'w> CpuQueryItem<'w> {
    pub fn operate(&mut self, operation: Op, addr: Option<u16>) -> bool {
        match operation {
            Op::XXX => self.nop(),
            Op::ADC => self.adc(addr),
            Op::AND => self.and(addr),
            Op::ASL => self.asl(addr),
            Op::BCC => self.bcc(addr),
            Op::BCS => self.bcs(addr),
            Op::BEQ => self.beq(addr),
            Op::BIT => self.bit(addr),
            Op::BMI => self.bmi(addr),
            Op::BNE => self.bne(addr),
            Op::BPL => self.bpl(addr),
            Op::BRK => self.brk(),
            Op::BVC => self.bvc(addr),
            Op::BVS => self.bvs(addr),
            Op::CLC => self.clc(),
            Op::CLD => self.cld(),
            Op::CLI => self.cli(),
            Op::CLV => self.clv(),
            Op::CMP => self.cmp(addr),
            Op::CPX => self.cpx(addr),
            Op::CPY => self.cpy(addr),
            Op::DEC => self.dec(addr),
            Op::DEX => self.dex(),
            Op::DEY => self.dey(),
            Op::EOR => self.eor(addr),
            Op::INC => self.inc(addr),
            Op::INX => self.inx(),
            Op::INY => self.iny(),
            Op::JMP => self.jmp(addr),
            Op::JSR => self.jsr(addr),
            Op::LDA => self.lda(addr),
            Op::LDX => self.ldx(addr),
            Op::LDY => self.ldy(addr),
            Op::LSR => self.lsr(addr),
            Op::NOP => self.nop(),
            Op::ORA => self.ora(addr),
            Op::PHA => self.pha(),
            Op::PHP => self.php(),
            Op::PLA => self.pla(),
            Op::PLP => self.plp(),
            Op::ROL => self.rol(addr),
            Op::ROR => self.ror(addr),
            Op::RTI => self.rti(),
            Op::RTS => self.rts(),
            Op::SBC => self.sbc(addr),
            Op::SEC => self.sec(),
            Op::SED => self.sed(),
            Op::SEI => self.sei(),
            Op::STA => self.sta(addr),
            Op::STX => self.stx(addr),
            Op::STY => self.sty(addr),
            Op::TAX => self.tax(),
            Op::TAY => self.tay(),
            Op::TSX => self.tsx(),
            Op::TXA => self.txa(),
            Op::TXS => self.txs(),
            Op::TYA => self.tya(),
        }
    }

    fn fetch(&mut self, addr: Option<u16>) -> u8 {
        addr.map(|addr| self.bus_read(addr)).unwrap_or(self.cpu.a)
    }

    fn write(&mut self, val: u8, addr: Option<u16>) {
        match addr {
            Some(addr) => self.bus_write(addr, val),
            None => self.cpu.a = val,
        }
    }

    pub fn and(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        self.cpu.a &= fetched;
        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);
        true
    }

    pub fn adc(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);

        let add_acc = self.cpu.a.overflowing_add(fetched);
        let result = add_acc.0.overflowing_add(self.cpu.status.0 & 0b1);

        self.cpu.status.set_carry(add_acc.1 || result.1);
        self.cpu.status.set_zero(result.0 == 0);
        self.cpu.status.set_negative(result.0 & 0x80 != 0);
        let overflow = (self.cpu.a ^ result.0) & !(self.cpu.a ^ fetched) & 0x80 != 0;
        self.cpu.status.set_overflow(overflow);

        self.cpu.a = result.0;
        true
    }

    pub fn sbc(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr) ^ 0xFF;

        let add_acc = self.cpu.a.overflowing_add(fetched);
        let result = add_acc.0.overflowing_add(self.cpu.status.0 & 0b1);

        self.cpu.status.set_carry(add_acc.1 || result.1);
        self.cpu.status.set_zero(result.0 == 0);
        self.cpu.status.set_negative(result.0 & 0x80 != 0);
        let overflow = (self.cpu.a ^ result.0) & !(self.cpu.a ^ fetched) & 0x80 != 0;
        self.cpu.status.set_overflow(overflow);

        self.cpu.a = result.0;
        true
    }

    pub fn bcc(&mut self, addr: Option<u16>) -> bool {
        self.branch(!self.cpu.status.carry(), addr);
        true
    }

    pub fn bcs(&mut self, addr: Option<u16>) -> bool {
        self.branch(self.cpu.status.carry(), addr);
        true
    }

    pub fn beq(&mut self, addr: Option<u16>) -> bool {
        self.branch(self.cpu.status.zero(), addr);
        true
    }

    pub fn bne(&mut self, addr: Option<u16>) -> bool {
        self.branch(!self.cpu.status.zero(), addr);
        true
    }

    pub fn bmi(&mut self, addr: Option<u16>) -> bool {
        self.branch(self.cpu.status.negative(), addr);
        true
    }

    pub fn bpl(&mut self, addr: Option<u16>) -> bool {
        self.branch(!self.cpu.status.negative(), addr);
        true
    }

    pub fn bvc(&mut self, addr: Option<u16>) -> bool {
        self.branch(!self.cpu.status.overflow(), addr);
        true
    }

    pub fn bvs(&mut self, addr: Option<u16>) -> bool {
        self.branch(self.cpu.status.overflow(), addr);
        true
    }

    fn branch(&mut self, val: bool, addr: Option<u16>) {
        if val {
            self.cpu.pc = self
                .cpu
                .pc
                .wrapping_add(addr.expect("no operand for relative branching"));
        }
    }

    pub fn brk(&mut self) -> bool {
        self.cpu.status.set_no_interrupt(true);
        let pc_bytes = self.cpu.pc.to_be_bytes();
        self.stack_push(pc_bytes[0]);
        self.stack_push(pc_bytes[1]);

        self.cpu.status.set_b_flag(true);
        self.stack_push(self.cpu.status.0);
        self.cpu.status.set_b_flag(false);

        self.cpu.pc = (self.bus_read(0xFFFE) as u16) | (self.bus_read(0xFFFF) as u16) << 8;
        false
    }

    pub fn clc(&mut self) -> bool {
        self.cpu.status.set_carry(false);
        false
    }

    pub fn cld(&mut self) -> bool {
        self.cpu.status.set_decimal(false);
        false
    }

    pub fn cli(&mut self) -> bool {
        self.cpu.status.set_no_interrupt(false);
        false
    }

    pub fn clv(&mut self) -> bool {
        self.cpu.status.set_overflow(false);
        false
    }

    pub fn asl(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        let result = fetched << 1;

        self.cpu.status.set_carry(fetched & 0x80 != 0);
        self.cpu.status.set_zero(result == 0);
        self.cpu.status.set_negative(result & 0x80 != 0);

        self.write(result, addr);

        false
    }

    pub fn lsr(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        let result = fetched >> 1;

        self.cpu.status.set_carry((fetched & 0x01) != 0);
        self.cpu.status.set_zero(result == 0);
        self.cpu.status.set_negative(false);

        self.write(result, addr);

        false
    }

    pub fn rol(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        let result = fetched << 1 | self.cpu.status.carry() as u8;

        self.cpu.status.set_carry(fetched & 0x80 != 0);
        self.cpu.status.set_zero(result == 0);
        self.cpu.status.set_negative(result & 0x80 != 0);

        self.write(result, addr);

        false
    }

    pub fn ror(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        let result = fetched >> 1 | (self.cpu.status.carry() as u8) << 7;

        self.cpu.status.set_carry(fetched & 0x1 != 0);
        self.cpu.status.set_zero(result == 0);
        self.cpu.status.set_negative(result & 0x80 != 0);

        self.write(result, addr);

        false
    }

    pub fn bit(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);

        let zero = fetched & self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        self.cpu.status.set_overflow(fetched & 0x40 != 0);
        self.cpu.status.set_negative(fetched & 0x80 != 0);

        false
    }

    pub fn cmp(&mut self, addr: Option<u16>) -> bool {
        self.compare(self.cpu.a, addr);
        true
    }

    pub fn cpx(&mut self, addr: Option<u16>) -> bool {
        self.compare(self.cpu.x, addr);
        true
    }

    pub fn cpy(&mut self, addr: Option<u16>) -> bool {
        self.compare(self.cpu.y, addr);
        true
    }

    fn compare(&mut self, val: u8, addr: Option<u16>) {
        let fetched = self.fetch(addr);
        self.cpu.status.set_carry(val >= fetched);
        self.cpu.status.set_zero(val == fetched);
        self.cpu
            .status
            .set_negative(val.wrapping_sub(fetched) & 0x80 != 0);
    }

    pub fn dec(&mut self, addr: Option<u16>) -> bool {
        let val = self.fetch(addr);
        let result = self.decrement(val);
        self.write(result, addr);
        false
    }

    pub fn dex(&mut self) -> bool {
        self.cpu.x = self.decrement(self.cpu.x);
        false
    }

    pub fn dey(&mut self) -> bool {
        self.cpu.y = self.decrement(self.cpu.y);
        false
    }

    fn decrement(&mut self, val: u8) -> u8 {
        let val = val.wrapping_sub(1);
        self.cpu.status.set_zero(val == 0);
        self.cpu.status.set_negative(val & 0x80 != 0);
        val
    }

    pub fn eor(&mut self, addr: Option<u16>) -> bool {
        let fetched = self.fetch(addr);
        self.cpu.a = fetched ^ self.cpu.a;

        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        true
    }

    pub fn inc(&mut self, addr: Option<u16>) -> bool {
        let val = self.fetch(addr);
        let result = self.increment(val);
        self.write(result, addr);
        false
    }

    pub fn inx(&mut self) -> bool {
        self.cpu.x = self.increment(self.cpu.x);
        false
    }

    pub fn iny(&mut self) -> bool {
        self.cpu.y = self.increment(self.cpu.y);
        false
    }

    fn increment(&mut self, val: u8) -> u8 {
        let val = val.wrapping_add(1);
        self.cpu.status.set_zero(val == 0);
        self.cpu.status.set_negative(val & 0x80 != 0);
        val
    }

    pub fn jmp(&mut self, addr: Option<u16>) -> bool {
        self.cpu.pc = addr.expect("no operand for jump");
        false
    }

    pub fn jsr(&mut self, addr: Option<u16>) -> bool {
        self.cpu.pc = self.cpu.pc.wrapping_sub(1);
        self.stack_push((self.cpu.pc >> 8 & 0x00FF) as u8);
        self.stack_push((self.cpu.pc & 0x00FF) as u8);

        self.cpu.pc = addr.expect("no operand for jsr");

        false
    }

    pub fn lda(&mut self, addr: Option<u16>) -> bool {
        self.cpu.a = self.fetch(addr);

        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        true
    }

    pub fn ldx(&mut self, addr: Option<u16>) -> bool {
        self.cpu.x = self.fetch(addr);

        let zero = self.cpu.x == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.x & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        true
    }

    pub fn ldy(&mut self, addr: Option<u16>) -> bool {
        self.cpu.y = self.fetch(addr);

        let zero = self.cpu.y == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.y & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        true
    }

    pub fn nop(&self) -> bool {
        false
    }

    pub fn ora(&mut self, addr: Option<u16>) -> bool {
        self.cpu.a = self.cpu.a | self.fetch(addr);

        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        true
    }

    pub fn pha(&mut self) -> bool {
        self.stack_push(self.cpu.a);
        false
    }

    pub fn php(&mut self) -> bool {
        self.stack_push(self.cpu.status.0 | 0x30);
        self.cpu.status.set_b_flag(false);
        false
    }

    pub fn pla(&mut self) -> bool {
        self.cpu.a = self.stack_pull();
        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);
        false
    }

    pub fn plp(&mut self) -> bool {
        self.cpu.status.0 = self.stack_pull() | 0x20;
        false
    }

    pub fn rti(&mut self) -> bool {
        self.cpu.status.0 = self.stack_pull() | 0x20;
        self.cpu.status.set_b_flag(false);

        self.cpu.pc = (self.stack_pull() as u16) | ((self.stack_pull() as u16) << 8);
        false
    }

    pub fn rts(&mut self) -> bool {
        self.cpu.pc = (self.stack_pull() as u16) | ((self.stack_pull() as u16) << 8);
        self.cpu.pc = self.cpu.pc.wrapping_add(1);
        false
    }

    pub fn sec(&mut self) -> bool {
        self.cpu.status.set_carry(true);
        false
    }

    pub fn sed(&mut self) -> bool {
        self.cpu.status.set_decimal(true);
        false
    }

    pub fn sei(&mut self) -> bool {
        self.cpu.status.set_no_interrupt(true);
        false
    }

    pub fn sta(&mut self, addr: Option<u16>) -> bool {
        self.write(self.cpu.a, addr);
        false
    }

    pub fn stx(&mut self, addr: Option<u16>) -> bool {
        self.write(self.cpu.x, addr);
        false
    }

    pub fn sty(&mut self, addr: Option<u16>) -> bool {
        self.write(self.cpu.y, addr);
        false
    }

    pub fn tax(&mut self) -> bool {
        self.cpu.x = self.cpu.a;

        let zero = self.cpu.x == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.x & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        false
    }

    pub fn tay(&mut self) -> bool {
        self.cpu.y = self.cpu.a;

        let zero = self.cpu.y == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.y & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        false
    }

    pub fn tsx(&mut self) -> bool {
        self.cpu.x = self.cpu.sp;

        let zero = self.cpu.x == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.x & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        false
    }

    pub fn txa(&mut self) -> bool {
        self.cpu.a = self.cpu.x;

        let zero = self.cpu.a == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.a & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        false
    }

    pub fn txs(&mut self) -> bool {
        self.cpu.sp = self.cpu.x;
        false
    }

    pub fn tya(&mut self) -> bool {
        self.cpu.a = self.cpu.y;

        let zero = self.cpu.y == 0;
        self.cpu.status.set_zero(zero);
        let negative = self.cpu.y & 0x80 != 0;
        self.cpu.status.set_negative(negative);

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cartridge::Cartridge,
        cpu::{CpuQuery, CpuQueryItem, CpuStatus},
        nes::NesBundle,
    };
    use bevy::prelude::*;

    macro_rules! setup {
        ($var:ident) => {
            let mut app = App::new();
            let cart = Cartridge::testing(None);
            app.world_mut().spawn((NesBundle::default(), cart));
            let mut query = app.world_mut().query::<CpuQuery>();
            let mut $var = query.single_mut(app.world_mut());
            $var.cpu.pc = 0x00;
        };
    }

    #[test]
    fn and() {
        setup!(query);
        let addr = Some(0x01);
        query.cpu.a = 0b0101_1100;

        query.bus.cpu_write(0x01, 0b1101_0110);

        query.and(addr);
        assert_eq!(query.cpu.a, 0b01010100);
        assert_eq!(query.cpu.status.0, 0b00100100);

        let addr = Some(0x02);

        query.and(addr);
        assert_eq!(query.cpu.a, 0b00000000);
        assert_eq!(query.cpu.status.0, 0b00100110);

        let addr = Some(0x01);
        query.cpu.a = 0b10001100;
        query.cpu.status = CpuStatus::default();

        query.and(addr);
        assert_eq!(query.cpu.a, 0b10000100);
        assert_eq!(query.cpu.status.0, 0b10100100);
    }

    #[test]
    fn adc() {
        setup!(query);
        let addr = Some(0x01);
        query.cpu.a = 34;
        query.bus.cpu_write(0x01, 56);

        query.adc(addr);
        assert_eq!(query.cpu.a, 90);
        assert_eq!(query.cpu.status.0, 0b00100100);

        query.cpu.a = 90;
        query.bus.cpu_write(0x01, 56);
        query.adc(addr);
        assert_eq!(query.cpu.a, 146);
        assert_eq!(query.cpu.status.0, 0b11100100);

        query.bus.cpu_write(0x01, 110);
        query.cpu.status = CpuStatus::default();
        query.adc(addr);
        assert_eq!(query.cpu.a, 0);
        assert_eq!(query.cpu.status.0, 0b00100111);

        query.cpu.status = CpuStatus::default();
        query.cpu.a = 135;
        query.bus.cpu_write(0x01, 145);
        query.adc(addr);
        assert_eq!(query.cpu.a, 24);
        assert_eq!(query.cpu.status.0, 0b01100101);
    }

    #[test]
    fn sbc() {
        setup!(query);
        let addr = Some(0x01);
        query.cpu.a = 34;

        query.bus.cpu_write(0x01, 56);

        query.sbc(addr);
        assert_eq!(query.cpu.a, 233);
        assert_eq!(query.cpu.status.0, 0b10100100);

        query.cpu.status = CpuStatus::default();
        query.cpu.a = 90;
        query.bus.cpu_write(0x01, 56);
        query.sbc(addr);
        assert_eq!(query.cpu.a, 33);
        assert_eq!(query.cpu.status.0, 0b00100101);

        query.bus.cpu_write(0x01, 110);
        query.cpu.status = CpuStatus::default();
        query.sbc(addr);
        assert_eq!(query.cpu.a, 178);
        assert_eq!(query.cpu.status.0, 0b10100100);

        query.cpu.status = CpuStatus::default();
        query.cpu.a = 135;
        query.bus.cpu_write(0x01, 19);
        query.sbc(addr);
        assert_eq!(query.cpu.a, 115);
        assert_eq!(query.cpu.status.0, 0b01100101);
    }

    #[test]
    fn bcc() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0x0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_carry(true);
        query.bcc(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_carry(false);
        query.bcc(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bcc(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bcs() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_carry(false);
        query.bcs(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_carry(true);
        query.bcs(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bcs(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn beq() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_zero(false);
        query.beq(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_zero(true);
        query.beq(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.beq(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bne() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_zero(true);
        query.bne(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_zero(false);
        query.bne(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bne(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bmi() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_negative(false);
        query.bmi(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_negative(true);
        query.bmi(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bmi(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bpl() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_negative(true);
        query.bpl(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_negative(false);
        query.bpl(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bpl(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bvc() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_overflow(true);
        query.bvc(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_overflow(false);
        query.bvc(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bvc(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn bvs() {
        setup!(query);
        let addr = 155;
        query.cpu.pc = 0;
        let addr = Some((addr as u16) | 0xFF00);

        query.cpu.status.set_overflow(false);
        query.bvs(addr);
        assert_eq!(query.cpu.pc, 0);

        query.cpu.status.set_overflow(true);
        query.bvs(addr);
        assert_eq!(query.cpu.pc, 65435);

        let addr = Some(13);
        query.bvs(addr);
        assert_eq!(query.cpu.pc, 65448);
    }

    #[test]
    fn asl() {
        setup!(query);
        query.cpu.pc = 0;
        query.cpu.a = 123;

        query.bus.cpu_write(0x01, 56);
        query.bus.cpu_write(0x02, 156);
        query.bus.cpu_write(0x03, 128);

        let addr: Option<u16> = None;
        test_asl(&mut query, 246, 0b1010_0100, addr);
        let addr = Some(0x00);
        test_asl(&mut query, 0, 0b0010_0110, addr);
        let addr = Some(0x01);
        test_asl(&mut query, 112, 0b0010_0100, addr);
        let addr = Some(0x02);
        test_asl(&mut query, 56, 0b0010_0101, addr);
        let addr = Some(0x03);
        test_asl(&mut query, 0, 0b0010_0111, addr);
    }

    fn test_asl(query: &mut CpuQueryItem, expect: u8, flags: u8, addr: Option<u16>) {
        assert!(!query.asl(addr));
        match addr {
            Some(addr) => assert_eq!(query.bus_read(addr), expect),
            None => assert_eq!(query.cpu.a, expect),
        }
        assert_eq!(
            query.cpu.status.0, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, query.cpu.status.0
        );
    }

    #[test]
    fn lsr() {
        setup!(query);
        query.cpu.pc = 0;
        query.cpu.a = 123;

        query.bus.cpu_write(0x01, 56);
        query.bus.cpu_write(0x02, 156);
        query.bus.cpu_write(0x03, 1);

        let addr: Option<u16> = None;
        test_lsr(&mut query, 61, 0b0010_0101, addr);
        let addr = Some(0x00);
        test_lsr(&mut query, 0, 0b0010_0110, addr);
        let addr = Some(0x01);
        test_lsr(&mut query, 28, 0b0010_0100, addr);
        let addr = Some(0x02);
        test_lsr(&mut query, 78, 0b0010_0100, addr);
        let addr = Some(0x03);
        test_lsr(&mut query, 0, 0b0010_0111, addr);
    }

    fn test_lsr(query: &mut CpuQueryItem, expect: u8, flags: u8, addr: Option<u16>) {
        assert!(!query.lsr(addr));
        match addr {
            Some(addr) => assert_eq!(query.bus_read(addr), expect),
            None => assert_eq!(query.cpu.a, expect),
        }
        assert_eq!(
            query.cpu.status.0, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, query.cpu.status.0
        );
    }

    #[test]
    fn rol() {
        setup!(query);
        query.cpu.pc = 0;
        query.cpu.a = 123;

        query.bus.cpu_write(0x01, 56);
        query.bus.cpu_write(0x02, 156);
        query.bus.cpu_write(0x03, 255);
        query.bus.cpu_write(0x04, 1);

        let addr: Option<u16> = None;
        test_rol(&mut query, 246, 0xA4, addr);
        let addr = Some(0x00);
        test_rol(&mut query, 0, 0x26, addr);
        let addr = Some(0x01);
        test_rol(&mut query, 112, 0x24, addr);
        let addr = Some(0x02);
        test_rol(&mut query, 56, 0x25, addr);
        let addr = Some(0x03);
        test_rol(&mut query, 255, 0xA5, addr);
        let addr = Some(0x04);
        test_rol(&mut query, 3, 0x24, addr);
    }

    fn test_rol(query: &mut CpuQueryItem, expect: u8, flags: u8, addr: Option<u16>) {
        assert!(!query.rol(addr));
        match addr {
            Some(addr) => assert_eq!(
                query.bus_read(addr),
                expect,
                "invalid addr {} => expected {} but was {}",
                addr,
                expect,
                query.bus_read(addr)
            ),
            None => assert_eq!(
                query.cpu.a, expect,
                "invalid accumulator => expected {} but was {}",
                expect, query.cpu.a
            ),
        }
        assert_eq!(
            query.cpu.status.0, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, query.cpu.status.0
        );
    }

    #[test]
    fn ror() {
        setup!(query);
        query.cpu.pc = 0;
        query.cpu.a = 123;

        query.bus.cpu_write(0x01, 56);
        query.bus.cpu_write(0x02, 156);
        query.bus.cpu_write(0x03, 255);
        query.bus.cpu_write(0x04, 37);

        let addr: Option<u16> = None;
        test_ror(&mut query, 61, 0x25, addr);
        let addr = Some(0x00);
        test_ror(&mut query, 128, 0xA4, addr);
        let addr = Some(0x01);
        test_ror(&mut query, 28, 0x24, addr);
        let addr = Some(0x02);
        test_ror(&mut query, 78, 0x24, addr);
        let addr = Some(0x03);
        test_ror(&mut query, 127, 0x25, addr);
        let addr = Some(0x04);
        test_ror(&mut query, 146, 0xA5, addr);
    }

    fn test_ror(query: &mut CpuQueryItem, expect: u8, flags: u8, addr: Option<u16>) {
        assert!(!query.ror(addr));
        match addr {
            Some(addr) => assert_eq!(
                query.bus_read(addr),
                expect,
                "invalid addr {} => expected {} but was {}",
                addr,
                expect,
                query.bus_read(addr)
            ),
            None => assert_eq!(
                query.cpu.a, expect,
                "invalid accumulator => expected {} but was {}",
                expect, query.cpu.a
            ),
        }
        assert_eq!(
            query.cpu.status.0, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, query.cpu.status.0
        );
    }

    #[test]
    fn bit() {
        setup!(query);
        query.cpu.pc = 0;
        query.cpu.a = 0b01101001;

        let addr = Some(0x00);
        query.bus.cpu_write(0x00, 0b10010110);
        test_bit(&mut query, 0b1010_0110, addr);

        let addr = Some(0x01);
        query.bus.cpu_write(0x01, 0b00110111);
        test_bit(&mut query, 0b0010_0100, addr);

        let addr = Some(0x02);
        query.bus.cpu_write(0x02, 0b11010000);
        test_bit(&mut query, 0b1110_0100, addr);
    }

    fn test_bit(query: &mut CpuQueryItem, flags: u8, addr: Option<u16>) {
        assert!(!query.bit(addr));
        assert_eq!(
            query.cpu.status.0, flags,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            flags, query.cpu.status.0
        );
    }

    #[test]
    fn brk() {
        setup!(query);
        query.bus.cpu_write(0xFFFE, 0x56);
        query.bus.cpu_write(0xFFFF, 0x78);

        query.cpu.status.0 = 0x82;
        query.cpu.pc = 0x1234;

        assert!(!query.brk());
        assert_eq!(query.cpu.pc, 0x7856);
        assert_eq!(
            query.cpu.status.0,
            0x82 | 0x04,
            "invalid flags => expected {:#010b}, but was {:#010b}",
            0x40,
            query.cpu.status.0
        );
    }

    #[test]
    fn clc() {
        setup!(query);
        query.cpu.status.0 = 0xFF;
        query.clc();
        assert_eq!(query.cpu.status.0, 0xFE);
    }

    #[test]
    fn cld() {
        setup!(query);
        query.cpu.status.0 = 0xFF;
        query.cld();
        assert_eq!(query.cpu.status.0, 0xF7);
    }

    #[test]
    fn cli() {
        setup!(query);
        query.cpu.status.0 = 0xFF;
        query.cli();
        assert_eq!(query.cpu.status.0, 0xFB);
    }

    #[test]
    fn clv() {
        setup!(query);
        query.cpu.status.0 = 0xFF;
        query.clv();
        assert_eq!(query.cpu.status.0, 0xBF);
    }

    #[test]
    fn cmp() {
        setup!(query);

        query.cpu.a = 0x56;
        let addr = Some(0x12);
        query.bus.cpu_write(0x12, 0x34);
        assert!(query.cmp(addr));
        assert_eq!(query.cpu.status.0, 0x25);

        query.cpu.a = 0x01;
        let addr = Some(0x13);
        query.bus.cpu_write(0x13, 0x34);
        assert!(query.cmp(addr));
        assert_eq!(query.cpu.status.0, 0xA4);

        query.cpu.a = 0x01;
        let addr = Some(0x14);
        query.bus.cpu_write(0x14, 0x01);
        assert!(query.cmp(addr));
        assert_eq!(query.cpu.status.0, 0x27);
    }

    #[test]
    fn cpx() {
        setup!(query);

        query.cpu.x = 0x56;
        let addr = Some(0x12);
        query.bus.cpu_write(0x12, 0x34);
        assert!(query.cpx(addr));
        assert_eq!(query.cpu.status.0, 0x25);

        query.cpu.x = 0x01;
        let addr = Some(0x13);
        query.bus.cpu_write(0x13, 0x34);
        assert!(query.cpx(addr));
        assert_eq!(query.cpu.status.0, 0xA4);

        query.cpu.x = 0x01;
        let addr = Some(0x14);
        query.bus.cpu_write(0x14, 0x01);
        assert!(query.cpx(addr));
        assert_eq!(query.cpu.status.0, 0x27);
    }

    #[test]
    fn cpy() {
        setup!(query);

        query.cpu.y = 0x56;
        let addr = Some(0x12);
        query.bus.cpu_write(0x12, 0x34);
        assert!(query.cpy(addr));
        assert_eq!(query.cpu.status.0, 0x25);

        query.cpu.y = 0x01;
        let addr = Some(0x13);
        query.bus.cpu_write(0x13, 0x34);
        assert!(query.cpy(addr));
        assert_eq!(query.cpu.status.0, 0xA4);

        query.cpu.y = 0x01;
        let addr = Some(0x14);
        query.bus.cpu_write(0x14, 0x01);
        assert!(query.cpy(addr));
        assert_eq!(query.cpu.status.0, 0x27);
    }

    #[test]
    fn dec() {
        setup!(query);

        let addr = Some(0x12);
        query.bus.cpu_write(0x12, 0x34);
        assert!(!query.dec(addr));
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.bus_read(0x12), 0x33);

        let addr = Some(0x13);
        query.bus.cpu_write(0x13, 0x84);
        assert!(!query.dec(addr));
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.bus_read(0x13), 0x83);

        let addr = Some(0x14);
        query.bus.cpu_write(0x14, 0x01);
        assert!(!query.dec(addr));
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.bus_read(0x14), 0x00);
    }

    #[test]
    fn dex() {
        setup!(query);

        query.cpu.x = 0x34;
        assert!(!query.dex());
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.cpu.x, 0x33);

        query.cpu.x = 0x84;
        assert!(!query.dex());
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.cpu.x, 0x83);

        query.cpu.x = 0x01;
        assert!(!query.dex());
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.cpu.x, 0x00);
    }

    #[test]
    fn dey() {
        setup!(query);

        query.cpu.y = 0x34;
        assert!(!query.dey());
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.cpu.y, 0x33);

        query.cpu.y = 0x84;
        assert!(!query.dey());
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.cpu.y, 0x83);

        query.cpu.y = 0x01;
        assert!(!query.dey());
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.cpu.y, 0x00);
    }

    #[test]
    fn eor() {
        setup!(query);

        let addr = Some(0x12);
        query.cpu.a = 0b11101110;
        query.bus.cpu_write(0x12, 0b01101010);
        assert!(query.eor(addr));
        assert_eq!(query.cpu.a, 0b10000100);
        assert_eq!(query.cpu.status.0, 0xA4);

        let addr = Some(0x12);
        query.cpu.a = 0b11110011;
        query.bus.cpu_write(0x12, 0b11110011);
        assert!(query.eor(addr));
        assert_eq!(query.cpu.a, 0b0);
        assert_eq!(query.cpu.status.0, 0x26);

        let addr = Some(0x12);
        query.cpu.a = 0b11110000;
        query.bus.cpu_write(0x12, 0b11110011);
        assert!(query.eor(addr));
        assert_eq!(query.cpu.a, 0b11);
        assert_eq!(query.cpu.status.0, 0x24);
    }

    #[test]
    fn inc() {
        setup!(query);

        let addr = Some(0x12);
        query.bus.cpu_write(0x12, 0x34);
        assert!(!query.inc(addr));
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.bus_read(0x12), 0x35);

        let addr = Some(0x13);
        query.bus.cpu_write(0x13, 0x84);
        assert!(!query.inc(addr));
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.bus_read(0x13), 0x85);

        let addr = Some(0x14);
        query.bus.cpu_write(0x14, 0xFF);
        assert!(!query.inc(addr));
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.bus_read(0x14), 0x00);
    }

    #[test]
    fn inx() {
        setup!(query);

        query.cpu.x = 0x34;
        assert!(!query.inx());
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.cpu.x, 0x35);

        query.cpu.x = 0x84;
        assert!(!query.inx());
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.cpu.x, 0x85);

        query.cpu.x = 0xFF;
        assert!(!query.inx());
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.cpu.x, 0x00);
    }

    #[test]
    fn iny() {
        setup!(query);

        query.cpu.y = 0x34;
        assert!(!query.iny());
        assert_eq!(query.cpu.status.0, 0x24);
        assert_eq!(query.cpu.y, 0x35);

        query.cpu.y = 0x84;
        assert!(!query.iny());
        assert_eq!(query.cpu.status.0, 0xA4);
        assert_eq!(query.cpu.y, 0x85);

        query.cpu.y = 0xFF;
        assert!(!query.iny());
        assert_eq!(query.cpu.status.0, 0x26);
        assert_eq!(query.cpu.y, 0x00);
    }

    #[test]
    fn jmp() {
        setup!(query);

        let addr = Some(0x1234);
        assert!(!query.jmp(addr));
        assert_eq!(query.cpu.pc, 0x1234);
    }

    #[test]
    fn jsr() {
        setup!(query);

        let addr = Some(0x1234);
        query.cpu.pc = 0x5678;
        query.cpu.sp = 0xFF;

        assert!(!query.jsr(addr));

        assert_eq!(query.bus_read(0x01FF), 0x56);
        assert_eq!(query.bus_read(0x01FE), 0x77);
        assert_eq!(query.cpu.pc, 0x1234);
    }

    #[test]
    fn lda() {
        setup!(query);

        let addr = Some(0x1234);
        query.bus.cpu_write(0x1234, 0x56);

        assert!(query.lda(addr));
        assert_eq!(query.cpu.a, 0x56);
    }

    #[test]
    fn ldx() {
        setup!(query);

        let addr = Some(0x1234);
        query.bus.cpu_write(0x1234, 0x56);

        assert!(query.ldx(addr));
        assert_eq!(query.cpu.x, 0x56);
    }

    #[test]
    fn ldy() {
        setup!(query);

        let addr = Some(0x1234);
        query.bus.cpu_write(0x1234, 0x56);

        assert!(query.ldy(addr));
        assert_eq!(query.cpu.y, 0x56);
    }

    #[test]
    fn pha() {
        setup!(query);

        query.cpu.a = 0x12;
        assert!(!query.pha());
        assert_eq!(query.bus_read(0x01FD), 0x12);

        query.cpu.a = 0x34;
        assert!(!query.pha());
        assert_eq!(query.bus_read(0x01FC), 0x34);
    }

    #[test]
    fn php() {
        setup!(query);

        query.cpu.status.0 = 0x12;
        assert!(!query.php());
        assert_eq!(query.bus_read(0x01FD), 0x32);

        query.cpu.status.0 = 0x34;
        assert!(!query.php());
        assert_eq!(query.bus_read(0x01FC), 0x34);
    }

    #[test]
    fn pla() {
        setup!(query);

        query.stack_push(0x12);
        query.stack_push(0x23);

        assert!(!query.pla());
        assert_eq!(query.cpu.a, 0x23);
        assert!(!query.pla());
        assert_eq!(query.cpu.a, 0x12);
    }

    #[test]
    fn plp() {
        setup!(query);

        query.stack_push(0x12);
        query.stack_push(0x23);

        assert!(!query.plp());
        assert_eq!(query.cpu.status.0, 0x23);
        assert!(!query.plp());
        assert_eq!(query.cpu.status.0, 0x32);
    }

    #[test]
    fn rti() {
        setup!(query);

        query.stack_push(0x12);
        query.stack_push(0x34);
        query.stack_push(0b10110000);

        assert!(!query.rti());
        assert_eq!(query.cpu.status.0, 0b10100000);
        assert_eq!(query.cpu.pc, 0x1234);
    }

    #[test]
    fn rts() {
        setup!(query);

        query.bus.cpu_write(0x01FF, 0x34);
        query.bus.cpu_write(0x01FE, 0x65);
        query.cpu.pc = 0x1234;
        query.cpu.sp = 0xFD;

        assert!(!query.rts());
        assert_eq!(query.cpu.pc, 0x3466);
    }

    #[test]
    fn sec() {
        setup!(query);

        assert!(!query.cpu.status.carry());
        assert!(!query.sec());
        assert!(query.cpu.status.carry());
    }

    #[test]
    fn sed() {
        setup!(query);

        assert!(!query.cpu.status.decimal());
        assert!(!query.sed());
        assert!(query.cpu.status.decimal());
    }

    #[test]
    fn sei() {
        setup!(query);

        assert!(!query.cli());
        assert!(!query.cpu.status.no_interrupt());
        assert!(!query.sei());
        assert!(query.cpu.status.no_interrupt());
    }

    #[test]
    fn sta() {
        setup!(query);

        query.cpu.a = 0x12;
        let addr = Some(0x6789);
        assert!(!query.sta(addr));
        assert_eq!(query.bus_read(0x6789), 0x12);
    }

    #[test]
    fn stx() {
        setup!(query);

        query.cpu.x = 0x12;
        let addr = Some(0x6789);

        assert!(!query.stx(addr));
        assert_eq!(query.bus_read(0x6789), 0x12);
    }

    #[test]
    fn sty() {
        setup!(query);

        query.cpu.y = 0x12;
        let addr = Some(0x6789);

        assert!(!query.sty(addr));
        assert_eq!(query.bus_read(0x6789), 0x12);
    }

    #[test]
    fn tax() {
        setup!(query);

        assert!(!query.tax());
        assert_eq!(query.cpu.x, 0x00);
        assert_eq!(query.cpu.status.0, 0x26);

        query.cpu.a = 0x23;
        assert!(!query.tax());
        assert_eq!(query.cpu.x, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.a = 0xB2;
        assert!(!query.tax());
        assert_eq!(query.cpu.x, 0xB2);
        assert_eq!(query.cpu.status.0, 0xA4);
    }

    #[test]
    fn tay() {
        setup!(query);

        assert!(!query.tay());
        assert_eq!(query.cpu.y, 0x00);
        assert_eq!(query.cpu.status.0, 0x26);

        query.cpu.a = 0x23;
        assert!(!query.tay());
        assert_eq!(query.cpu.y, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.a = 0xB2;
        assert!(!query.tay());
        assert_eq!(query.cpu.y, 0xB2);
        assert_eq!(query.cpu.status.0, 0xA4);
    }

    #[test]
    fn tsx() {
        setup!(query);

        query.cpu.sp = 0x00;
        assert!(!query.tsx());
        assert_eq!(query.cpu.x, 0x00);
        assert_eq!(query.cpu.status.0, 0x26);

        query.cpu.sp = 0x23;
        assert!(!query.tsx());
        assert_eq!(query.cpu.x, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.sp = 0xB2;
        assert!(!query.tsx());
        assert_eq!(query.cpu.x, 0xB2);
        assert_eq!(query.cpu.status.0, 0xA4);
    }

    #[test]
    fn txa() {
        setup!(query);

        assert!(!query.txa());
        assert_eq!(query.cpu.a, 0x00);
        assert_eq!(query.cpu.status.0, 0x26);

        query.cpu.x = 0x23;
        assert!(!query.txa());
        assert_eq!(query.cpu.a, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.x = 0xB2;
        assert!(!query.txa());
        assert_eq!(query.cpu.a, 0xB2);
        assert_eq!(query.cpu.status.0, 0xA4);
    }

    #[test]
    fn txs() {
        setup!(query);

        assert!(!query.txs());
        assert_eq!(query.cpu.sp, 0x00);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.x = 0x23;
        assert!(!query.txs());
        assert_eq!(query.cpu.sp, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.x = 0xB2;
        assert!(!query.txs());
        assert_eq!(query.cpu.sp, 0xB2);
        assert_eq!(query.cpu.status.0, 0x24);
    }

    #[test]
    fn tya() {
        setup!(query);

        assert!(!query.tya());
        assert_eq!(query.cpu.a, 0x00);
        assert_eq!(query.cpu.status.0, 0x26);

        query.cpu.y = 0x23;
        assert!(!query.tya());
        assert_eq!(query.cpu.a, 0x23);
        assert_eq!(query.cpu.status.0, 0x24);

        query.cpu.y = 0xB2;
        assert!(!query.tya());
        assert_eq!(query.cpu.a, 0xB2);
        assert_eq!(query.cpu.status.0, 0xA4);
    }
}
