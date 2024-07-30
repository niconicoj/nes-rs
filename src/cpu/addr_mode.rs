use super::CpuQueryItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    _XXX,
}

impl<'w> CpuQueryItem<'w> {
    pub fn addr_mode(&mut self) -> (Option<u16>, bool) {
        match self.cpu.addr_mode {
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
            AddrMode::_XXX => self.imp(),
        }
    }

    fn imp(&mut self) -> (Option<u16>, bool) {
        (None, false)
    }

    fn imm(&mut self) -> (Option<u16>, bool) {
        (Some(self.cpu.adv()), false)
    }

    fn acc(&mut self) -> (Option<u16>, bool) {
        (None, false)
    }

    fn rel(&mut self) -> (Option<u16>, bool) {
        let addr = self.cpu.adv();
        let addr = self.bus_read(addr);
        // if the number is negative, i.e. it has its 7th bit set,
        // then we 'or' it with 0xFF00 so that the math checks out later
        let addr = if addr & 0x80 != 0x00 {
            (addr as u16) | 0xFF00
        } else {
            addr as u16
        };
        let page_crossed = (addr.wrapping_add(self.cpu.pc) & 0xFF00) != (self.cpu.pc & 0xFF00);
        (Some(addr), page_crossed)
    }

    // absolute addressing mode
    fn abs(&mut self) -> (Option<u16>, bool) {
        let lsb = self.cpu.adv();
        let msb = self.cpu.adv();
        (
            Some((self.bus_read(lsb) as u16) | ((self.bus_read(msb) as u16) << 8)),
            false,
        )
    }

    fn abx(&mut self) -> (Option<u16>, bool) {
        let lsb_addr = self.cpu.adv();
        let msb_addr = self.cpu.adv();
        let lsb = self.bus_read(lsb_addr) as u16;
        let msb = self.bus_read(msb_addr) as u16;
        let addr = lsb | (msb << 8);
        let abx_addr = addr.wrapping_add(self.cpu.x as u16);
        if abx_addr & 0xFF00 != (msb << 8) {
            (Some(abx_addr), true)
        } else {
            (Some(abx_addr), false)
        }
    }

    fn aby(&mut self) -> (Option<u16>, bool) {
        let lsb = self.cpu.adv();
        let msb = self.cpu.adv();
        let addr = (self.bus_read(lsb) as u16) | (self.bus_read(msb) as u16) << 8;
        let aby_addr = addr.wrapping_add(self.cpu.y as u16);
        if aby_addr & 0xFF00 != addr & 0xFF00 {
            (Some(aby_addr), true)
        } else {
            (Some(aby_addr), false)
        }
    }

    // zero page addressing mode
    fn zp0(&mut self) -> (Option<u16>, bool) {
        let addr = self.cpu.adv();
        (Some(self.bus_read(addr) as u16), false)
    }

    fn zpx(&mut self) -> (Option<u16>, bool) {
        let addr = self.cpu.adv();
        (
            Some(self.bus_read(addr).wrapping_add(self.cpu.x) as u16),
            false,
        )
    }

    fn zpy(&mut self) -> (Option<u16>, bool) {
        let addr = self.cpu.adv();
        (
            Some(self.bus_read(addr).wrapping_add(self.cpu.y) as u16),
            false,
        )
    }

    // indirect addressing modes

    fn ind(&mut self) -> (Option<u16>, bool) {
        let lsb = self.cpu.adv();
        let msb = self.cpu.adv();
        let ptr = (self.bus_read(lsb) as u16) | ((self.bus_read(msb) as u16) << 8);
        if ptr & 0x00FF == 0x00FF {
            (
                Some(((self.bus_read(ptr & 0xFF00) as u16) << 8) | self.bus_read(ptr) as u16),
                false,
            )
        } else {
            (
                Some(((self.bus_read(ptr + 1) as u16) << 8) | self.bus_read(ptr) as u16),
                false,
            )
        }
    }

    fn idx(&mut self) -> (Option<u16>, bool) {
        // get the self.bus.value pointed at by the program counter
        // then, add the x register to that value
        let addr = self.cpu.adv();
        let addr = self.bus_read(addr).wrapping_add(self.cpu.x) as u16;
        // the above value is then used as the zero paged address of
        // the pointer address that we want to use
        (
            Some(
                (self.bus_read((addr + 1) & 0x00FF) as u16) << 8
                    | self.bus_read(addr & 0x00FF) as u16,
            ),
            false,
        )
    }

    fn idy(&mut self) -> (Option<u16>, bool) {
        let addr = self.cpu.adv();
        let addr = self.bus_read(addr);

        let lo = self.bus_read(addr as u16) as u16;
        let hi = self.bus_read(addr.wrapping_add(1) as u16) as u16;

        let idy_addr = (hi << 8 | lo).wrapping_add(self.cpu.y as u16);

        if idy_addr & 0xFF00 != (hi << 8) {
            (Some(idy_addr), true)
        } else {
            (Some(idy_addr), false)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{cpu::CpuQuery, nes::NesBundle};
    use bevy::prelude::*;

    macro_rules! setup {
        ($var:ident) => {
            let mut app = App::new();
            app.world_mut().spawn(NesBundle::default());
            let mut query = app.world_mut().query::<CpuQuery>();
            let mut $var = query.single_mut(app.world_mut());
            $var.cpu.pc = 0x00;
        };
    }

    #[test]
    fn acc() {
        setup!(query);

        assert_eq!(query.acc(), (None, false));
    }

    #[test]
    fn imp() {
        setup!(query);

        assert_eq!(query.imp(), (None, false));
    }

    #[test]
    fn imm() {
        setup!(query);

        assert_eq!(query.imm(), (Some(0x00), false));
        assert_eq!(query.imm(), (Some(0x01), false));
    }

    #[test]
    fn rel() {
        setup!(query);

        query.bus_write(0x0, 0x34);
        assert_eq!(query.rel(), (Some(0x34), false));

        query.cpu.pc = 0x34;
        query.bus_write(0x34, 0xFB);
        assert_eq!(query.rel(), (Some(0xFFFB), false));
    }

    #[test]
    fn abs() {
        setup!(query);

        query.bus_write(0x0, 0x34);
        query.bus_write(0x1, 0x4A);
        assert_eq!(query.abs(), (Some(0x4A34), false));
        assert_eq!(query.cpu.pc, 0x02);

        query.cpu.pc = 0x34;
        query.bus_write(0x34, 0xFB);
        query.bus_write(0x35, 0x12);
        assert_eq!(query.abs(), (Some(0x12FB), false));
        assert_eq!(query.cpu.pc, 0x36);
    }

    #[test]
    fn abx() {
        setup!(query);

        query.cpu.x = 0x45;
        query.bus_write(0x0, 0x34);
        query.bus_write(0x1, 0x00);
        assert_eq!(query.abx(), (Some(0x0079), false));
        assert_eq!(query.cpu.pc, 0x2);

        query.cpu.pc = 0x1034;
        query.cpu.x = 0xFC;
        query.bus_write(0x1034, 0x0B);
        query.bus_write(0x1035, 0xFF);
        assert_eq!(query.abx(), (Some(0x0007), true));
        assert_eq!(query.cpu.pc, 0x1036);
    }

    #[test]
    fn aby() {
        setup!(query);

        query.cpu.y = 0x45;
        query.bus_write(0x0, 0x34);
        query.bus_write(0x1, 0x3A);
        assert_eq!(query.aby(), (Some(0x3A79), false));
        assert_eq!(query.cpu.pc, 0x2);

        query.cpu.pc = 0x34;
        query.cpu.y = 0xFC;
        query.bus_write(0x34, 0x0B);
        query.bus_write(0x35, 0xFF);
        assert_eq!(query.aby(), (Some(0x0007), true));
        assert_eq!(query.cpu.pc, 0x36);
    }

    #[test]
    fn zp0() {
        setup!(query);

        query.bus_write(0x00, 0x35);
        assert_eq!(query.zp0(), (Some(0x0035), false));
        assert_eq!(query.cpu.pc, 0x01);

        query.cpu.pc = 0x56;
        query.bus_write(0x56, 0xEF);
        assert_eq!(query.zp0(), (Some(0x00EF), false));
        assert_eq!(query.cpu.pc, 0x57);
    }

    #[test]
    fn zpx() {
        setup!(query);

        query.cpu.x = 0xA1;
        query.bus_write(0x00, 0x35);
        assert_eq!(query.zpx(), (Some(0x00D6), false));
        assert_eq!(query.cpu.pc, 0x01);

        query.cpu.pc = 0x56;
        query.cpu.x = 0x3D;
        query.bus_write(0x56, 0xEF);
        assert_eq!(query.zpx(), (Some(0x002C), false));
        assert_eq!(query.cpu.pc, 0x57);
    }

    #[test]
    fn zpy() {
        setup!(query);

        query.cpu.y = 0xA1;
        query.bus_write(0x00, 0x35);
        assert_eq!(query.zpy(), (Some(0x00D6), false));
        assert_eq!(query.cpu.pc, 0x01);

        query.cpu.pc = 0x56;
        query.cpu.y = 0x3D;
        query.bus_write(0x56, 0xEF);
        assert_eq!(query.zpy(), (Some(0x002C), false));
        assert_eq!(query.cpu.pc, 0x57);
    }

    #[test]
    fn ind() {
        setup!(query);

        query.bus_write(0x00, 0x35);
        query.bus_write(0x01, 0x0A);
        query.bus_write(0x0A35, 0x34);
        query.bus_write(0x0A36, 0x12);
        assert_eq!(query.ind(), (Some(0x1234), false));
        assert_eq!(query.cpu.pc, 0x02);

        query.cpu.pc = 0x56;
        query.bus_write(0x56, 0x12);
        query.bus_write(0x57, 0x10);
        query.bus_write(0x1012, 0x12);
        query.bus_write(0x1013, 0xFE);
        assert_eq!(query.ind(), (Some(0xFE12), false));
        assert_eq!(query.cpu.pc, 0x58);

        // test for the indirect addr mode bug
        // when pointing to an address of the form 0xXXFF
        // instead of reading into the next page it wraps to
        // the first byte of the current page
        query.cpu.pc = 0x12;
        query.bus_write(0x12, 0xFF);
        query.bus_write(0x13, 0x12);
        query.bus_write(0x12FF, 0x34);
        query.bus_write(0x1200, 0x12);
        assert_eq!(query.ind(), (Some(0x1234), false));
        assert_eq!(query.cpu.pc, 0x14);
    }

    #[test]
    fn idx() {
        setup!(query);

        query.cpu.x = 0x1A;
        query.bus_write(0x00, 0x2B);
        query.bus_write(0x45, 0x34);
        query.bus_write(0x46, 0x12);
        assert_eq!(query.idx(), (Some(0x1234), false));
        assert_eq!(query.cpu.pc, 0x01);

        query.cpu.pc = 0x12;
        query.cpu.x = 0xE3;
        query.bus_write(0x12, 0x2B);
        query.bus_write(0x0E, 0x34);
        query.bus_write(0x0F, 0x12);
        assert_eq!(query.idx(), (Some(0x1234), false));
        assert_eq!(query.cpu.pc, 0x13);
    }

    #[test]
    fn idy() {
        setup!(query);

        query.cpu.y = 0x1A;
        query.bus_write(0x00, 0x2B);
        query.bus_write(0x2B, 0x34);
        query.bus_write(0x2C, 0x12);
        assert_eq!(query.idy(), (Some(0x124E), false));
        assert_eq!(query.cpu.pc, 0x01);

        query.cpu.pc = 0x14;
        query.cpu.y = 0x01;
        query.bus_write(0x14, 0xAB);
        query.bus_write(0x00AB, 0xFF);
        query.bus_write(0x00AC, 0x12);
        assert_eq!(query.idy(), (Some(0x1300), true));
        assert_eq!(query.cpu.pc, 0x15);

        query.cpu.pc = 0x16;
        query.cpu.y = 0x00;
        query.bus_write(0x16, 0xFF);
        query.bus_write(0x00FF, 0xAD);
        query.bus_write(0x0000, 0xDE);
        assert_eq!(query.idy(), (Some(0xDEAD), false));
        assert_eq!(query.cpu.pc, 0x17);
    }
}
