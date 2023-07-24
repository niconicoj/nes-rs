use std::cmp::Ordering;

use crate::{ram::Ram, bus::BusDevice};

pub struct NRom128 {
    /// PRG RAM goes for 0x0000 to 0x1FFF
    /// PRG ROM goes from 0x2000 to 0x5FFF
    data: [u8; 0x6000],
}

impl BusDevice for NRom128 {
    fn addr_space(&self) -> usize {
        self.data.len()
    }

    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data;
    }
}

pub struct NRom256 {
    /// PRG RAM goes for 0x0000 to 0x1FFF
    /// PRG ROM goes from 0x2000 to 0x9FFF
    data: [u8; 0xA000],
}

impl BusDevice for NRom256 {
    fn addr_space(&self) -> usize {
        self.data.len()
    }

    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data;
    }
}
