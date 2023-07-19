use crate::ram::Ram;

#[derive(Default)]
pub struct Bus {
    ram: Ram<0xFFFF>,
}

impl Bus {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0xFFFF => self.ram.read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram.write(addr, data)
    }
}
