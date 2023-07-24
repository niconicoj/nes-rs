use super::Cpu;

#[derive(Debug, Clone, Copy)]
pub enum AddrMode {
    IMP,
    IMM,
    ACC,
    REL,
    ABS,
    ABX,
    ABY,
    ZP0,
    ZPX,
    ZPY,
    IND,
    IDX,
    IDY,
    XXX,
}

impl Cpu {
    pub fn addr_mode(&mut self, addr: AddrMode) -> bool {
        println!("using addressing mode {:?}", addr);
        match addr {
            AddrMode::IMP => self.imp(),
            AddrMode::IMM => self.imm(),
            AddrMode::ACC => self.acc(),
            AddrMode::REL => self.rel(),
            AddrMode::ABS => self.abs(),
            AddrMode::ABX => self.abx(),
            AddrMode::ABY => self.aby(),
            AddrMode::ZP0 => self.zp0(),
            AddrMode::ZPX => self.zpx(),
            AddrMode::ZPY => self.zpy(),
            AddrMode::IND => self.ind(),
            AddrMode::IDX => self.idx(),
            AddrMode::IDY => self.idy(),
            AddrMode::XXX => self.imp(),
        }
    }

    #[inline]
    fn imp(&mut self) -> bool {
        self.op_addr = None;
        false
    }

    #[inline]
    fn imm(&mut self) -> bool {
        self.op_addr = Some(self.prg_cntr.adv());
        false
    }

    #[inline]
    fn acc(&mut self) -> bool {
        self.op_addr = None;
        false
    }

    #[inline]
    fn rel(&mut self) -> bool {
        let addr = self.bus.read(self.prg_cntr.adv());
        // if the number is negative, i.e. it has its 7th bit set,
        // then we 'or' it with 0xFF00 so that the math checks out later
        if addr & 0x80 != 0x00 {
            self.op_addr = Some((addr as u16) | 0xFF00);
        } else {
            self.op_addr = Some(addr as u16);
        }
        false
    }

    // absolute addressing mode

    #[inline]
    fn abs(&mut self) -> bool {
        self.op_addr = Some(
            (self.bus.read(self.prg_cntr.adv()) as u16)
                | (self.bus.read(self.prg_cntr.adv()) as u16) << 8,
        );
        false
    }

    #[inline]
    fn abx(&mut self) -> bool {
        let addr = (self.bus.read(self.prg_cntr.adv()) as u16)
            | (self.bus.read(self.prg_cntr.adv()) as u16) << 8;

        self.op_addr = Some(addr.wrapping_add(self.x as u16));
        if unsafe { self.op_addr.unwrap_unchecked() & 0xFF00 != addr & 0xFF00 } {
            true
        } else {
            false
        }
    }

    #[inline]
    fn aby(&mut self) -> bool {
        let addr = (self.bus.read(self.prg_cntr.adv()) as u16)
            | (self.bus.read(self.prg_cntr.adv()) as u16) << 8;

        self.op_addr = Some(addr.wrapping_add(self.y as u16));
        if unsafe { self.op_addr.unwrap_unchecked() & 0xFF00 != addr & 0xFF00 } {
            true
        } else {
            false
        }
    }

    // zero page addressing mode

    #[inline]
    fn zp0(&mut self) -> bool {
        self.op_addr = Some(self.bus.read(self.prg_cntr.adv()) as u16);
        false
    }

    #[inline]
    fn zpx(&mut self) -> bool {
        self.op_addr = Some(self.bus.read(self.prg_cntr.adv()).wrapping_add(self.x) as u16);
        false
    }

    #[inline]
    fn zpy(&mut self) -> bool {
        self.op_addr = Some(self.bus.read(self.prg_cntr.adv()).wrapping_add(self.y) as u16);
        false
    }

    // indirect addressing modes

    #[inline]
    fn ind(&mut self) -> bool {
        let ptr = (self.bus.read(self.prg_cntr.adv()) as u16)
            | (self.bus.read(self.prg_cntr.adv()) as u16) << 8;
        if ptr & 0x00FF == 0x00FF {
            self.op_addr =
                Some((self.bus.read(ptr & 0xFF00) as u16) << 8 | self.bus.read(ptr + 0) as u16);
        } else {
            self.op_addr =
                Some((self.bus.read(ptr + 1) as u16) << 8 | self.bus.read(ptr + 0) as u16);
        }
        false
    }

    #[inline]
    fn idx(&mut self) -> bool {
        // get the bus value pointed at by the program counter
        // then, add the x register to that value
        let addr = self.bus.read(self.prg_cntr.adv()).wrapping_add(self.x) as u16;
        // the above value is then used as the zero paged address of
        // the pointer address that we want to use
        self.op_addr = Some(
            (self.bus.read((addr + 1) & 0x00FF) as u16) << 8 | self.bus.read(addr & 0x00FF) as u16,
        );
        false
    }

    #[inline]
    fn idy(&mut self) -> bool {
        let addr = self.bus.read(self.prg_cntr.adv()) as u16;

        let ptr = (self.bus.read(addr) as u16) | (self.bus.read(addr + 1) as u16) << 8;
        println!("unaltered ptr : {}", ptr);

        self.op_addr = Some(ptr.wrapping_add(self.y as u16));
        println!("altered ptr : {}", self.op_addr.unwrap());
        if unsafe { self.op_addr.unwrap_unchecked() & 0xFF00 != ptr & 0xFF00 } {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{bus::Bus, cpu::ProgramCounter};

    #[test]
    fn acc() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        assert!(!cpu.acc());
        assert_eq!(cpu.op_addr, None);
    }

    #[test]
    fn imp() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        assert!(!cpu.imp());
        assert_eq!(cpu.op_addr, None);
    }

    #[test]
    fn imm() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        assert!(!cpu.imm());
        assert_eq!(cpu.op_addr, Some(0x0));
        assert!(!cpu.imm());
        assert_eq!(cpu.op_addr, Some(0x1));
    }

    #[test]
    fn rel() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.bus.write(0x0, 0x34);

        assert!(!cpu.rel());
        assert_eq!(cpu.op_addr, Some(0x34));

        cpu.prg_cntr = ProgramCounter(0x34);
        cpu.bus.write(0x34, 0xFB);

        assert!(!cpu.rel());
        assert_eq!(cpu.op_addr, Some(0xFFFB));
    }

    #[test]
    fn abs() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.bus.write(0x0, 0x34);
        cpu.bus.write(0x1, 0x3A);

        assert!(!cpu.abs());
        assert_eq!(cpu.op_addr, Some(0x3A34));
        assert_eq!(cpu.prg_cntr.0, 0x02);

        cpu.prg_cntr = ProgramCounter(0x34);
        cpu.bus.write(0x34, 0xFB);
        cpu.bus.write(0x35, 0x12);

        assert!(!cpu.abs());
        assert_eq!(cpu.op_addr, Some(0x12FB));
        assert_eq!(cpu.prg_cntr.0, 0x36);
    }

    #[test]
    fn abx() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.x = 0x45;
        cpu.bus.write(0x0, 0x34);
        cpu.bus.write(0x1, 0x3A);

        assert!(!cpu.abx());
        assert_eq!(cpu.op_addr, Some(0x3A79));
        assert_eq!(cpu.prg_cntr.0, 0x2);

        cpu.prg_cntr = ProgramCounter(0x34);
        cpu.x = 0xFC;
        cpu.bus.write(0x34, 0x0B);
        cpu.bus.write(0x35, 0xFF);

        assert!(cpu.abx());
        assert_eq!(cpu.op_addr, Some(0x0007));
        assert_eq!(cpu.prg_cntr.0, 0x36);
    }

    #[test]
    fn aby() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.y = 0x45;
        cpu.bus.write(0x0, 0x34);
        cpu.bus.write(0x1, 0x3A);

        assert!(!cpu.aby());
        assert_eq!(cpu.op_addr, Some(0x3A79));
        assert_eq!(cpu.prg_cntr.0, 0x2);

        cpu.prg_cntr = ProgramCounter(0x34);
        cpu.y = 0xFC;
        cpu.bus.write(0x34, 0x0B);
        cpu.bus.write(0x35, 0xFF);

        assert!(cpu.aby());
        assert_eq!(cpu.op_addr, Some(0x0007));
        assert_eq!(cpu.prg_cntr.0, 0x36);
    }

    #[test]
    fn zp0() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.bus.write(0x00, 0x35);

        assert!(!cpu.zp0());
        assert_eq!(cpu.op_addr, Some(0x0035));
        assert_eq!(cpu.prg_cntr.0, 0x01);

        cpu.prg_cntr = ProgramCounter(0x56);
        cpu.bus.write(0x56, 0xEF);

        assert!(!cpu.zp0());
        assert_eq!(cpu.op_addr, Some(0x00EF));
        assert_eq!(cpu.prg_cntr.0, 0x57);
    }

    #[test]
    fn zpx() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.x = 0xA1;
        cpu.bus.write(0x00, 0x35);

        assert!(!cpu.zpx());
        assert_eq!(cpu.op_addr, Some(0x00D6));
        assert_eq!(cpu.prg_cntr.0, 0x01);

        cpu.prg_cntr = ProgramCounter(0x56);
        cpu.x = 0x3D;
        cpu.bus.write(0x56, 0xEF);

        assert!(!cpu.zpx());
        assert_eq!(cpu.op_addr, Some(0x002C));
        assert_eq!(cpu.prg_cntr.0, 0x57);
    }

    #[test]
    fn zpy() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.y = 0xA1;
        cpu.bus.write(0x00, 0x35);

        assert!(!cpu.zpy());
        assert_eq!(cpu.op_addr, Some(0x00D6));
        assert_eq!(cpu.prg_cntr.0, 0x01);

        cpu.prg_cntr = ProgramCounter(0x56);
        cpu.y = 0x3D;
        cpu.bus.write(0x56, 0xEF);

        assert!(!cpu.zpy());
        assert_eq!(cpu.op_addr, Some(0x002C));
        assert_eq!(cpu.prg_cntr.0, 0x57);
    }

    #[test]
    fn ind() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.bus.write(0x00, 0x35);
        cpu.bus.write(0x01, 0xD6);
        cpu.bus.write(0xD635, 0x34);
        cpu.bus.write(0xD636, 0x12);

        assert!(!cpu.ind());
        assert_eq!(cpu.op_addr, Some(0x1234));
        assert_eq!(cpu.prg_cntr.0, 0x02);

        cpu.prg_cntr = ProgramCounter(0x56);
        cpu.bus.write(0x56, 0x12);
        cpu.bus.write(0x57, 0xFE);
        cpu.bus.write(0xFE12, 0x12);
        cpu.bus.write(0xFE13, 0xFE);

        assert!(!cpu.ind());
        assert_eq!(cpu.op_addr, Some(0xFE12));
        assert_eq!(cpu.prg_cntr.0, 0x58);

        // test for the indirect addr mode bug
        // when pointing to an address of the form 0xXXFF
        // instead of reading into the next page it wraps to
        // the first byte of the current page

        cpu.prg_cntr = ProgramCounter(0x12);
        cpu.bus.write(0x12, 0xFF);
        cpu.bus.write(0x13, 0xD6);
        cpu.bus.write(0xD6FF, 0x34);
        cpu.bus.write(0xD600, 0x12);

        assert!(!cpu.ind());
        assert_eq!(cpu.op_addr, Some(0x1234));
        assert_eq!(cpu.prg_cntr.0, 0x14);
    }

    #[test]
    fn idx() {
        let mut bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.x = 0x1A;
        cpu.bus.write(0x00, 0x2B);
        cpu.bus.write(0x45, 0x34);
        cpu.bus.write(0x46, 0x12);

        assert!(!cpu.idx());
        assert_eq!(cpu.op_addr, Some(0x1234));
        assert_eq!(cpu.prg_cntr.0, 0x01);

        cpu.prg_cntr = ProgramCounter(0x12);
        cpu.x = 0xE3;
        cpu.bus.write(0x12, 0x2B);
        cpu.bus.write(0x0E, 0x34);
        cpu.bus.write(0x0F, 0x12);

        assert!(!cpu.idx());
        assert_eq!(cpu.op_addr, Some(0x1234));
        assert_eq!(cpu.prg_cntr.0, 0x13);
    }

    #[test]
    fn idy() {
        let bus = Bus::ram_only();
        let mut cpu = Cpu::new(bus);
        cpu.y = 0x1A;
        cpu.bus.write(0x00, 0x2B);
        cpu.bus.write(0x2B, 0x34);
        cpu.bus.write(0x2C, 0x12);

        assert!(!cpu.idy());
        assert_eq!(cpu.op_addr, Some(0x124E));
        assert_eq!(cpu.prg_cntr.0, 0x01);

        cpu.prg_cntr = ProgramCounter(0x12);
        cpu.y = 0xB6;
        cpu.bus.write(0x12, 0x64);
        cpu.bus.write(0x64, 0x86);
        cpu.bus.write(0x65, 0x12);

        assert!(cpu.idy());
        assert_eq!(cpu.op_addr, Some(0x133C));
        assert_eq!(cpu.prg_cntr.0, 0x13);
    }
}
