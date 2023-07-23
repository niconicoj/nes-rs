use std::ops::Range;

use crate::ram::Ram;

pub trait BusDevice {
    fn addr_space(&self) -> usize;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug)]
pub enum BusError {
    OverlappingDevice,
    OutOfBoundDevice,
}
#[derive(Default)]
pub struct Bus {
    devices: Vec<(usize, Box<dyn BusDevice>)>,
}

impl Bus {
    /// returns a Bus where every adresses is just RAM.
    /// Useful for testing cpu.
    pub fn ram_only() -> Self {
        let mut bus = Bus::default();
        bus.plug(Box::new(Ram::<0x10000>::default()), 0)
            .expect("failed to create ram only bus");
        bus
    }

    /// plug a new device onto the bus. Errors out if the device address range does not fit or overlaps with an
    /// already plugged device.
    pub fn plug(&mut self, device: Box<dyn BusDevice>, offset: usize) -> Result<(), BusError> {
        // check if the device fits in the intended location
        if let Some(addr_end) = device.addr_space().checked_add(offset) {
            if self.devices.iter().any(|(curr_offset, device)| {
                *curr_offset <= addr_end && device.addr_space() + *curr_offset >= offset
            }) {
                return Err(BusError::OverlappingDevice);
            };
            self.devices.push((offset, device));
        } else {
            return Err(BusError::OutOfBoundDevice);
        }

        Ok(())
    }

    pub fn read(&self, addr: u16) -> u8 {
        if let Some((offset, device)) = self.devices.iter().find(|(offset, device)| {
            (*offset..(offset + device.addr_space())).contains(&(addr as usize))
        }) {
            device.read(addr - (*offset as u16))
        } else {
            println!("warn: reading from open bus");
            0
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if let Some((offset, device)) = self.devices.iter_mut().find(|(offset, device)| {
            (*offset..(offset + device.addr_space())).contains(&(addr as usize))
        }) {
            device.write(addr - (*offset as u16), data);
        }
    }
}

mod test {
    use crate::{bus::Bus, ram::Ram};

    #[test]
    fn plug() {
        let mut bus = Bus::default();

        bus.plug(Box::new(Ram::<0x800>::new_zeroed(0x2000)), 0x0)
            .expect("failed to plug ram");

        bus.write(0x0001, 0xAA);
        assert_eq!(bus.read(0x0001), 0xAA);
        assert_eq!(bus.read(0x0801), 0xAA);
        assert_eq!(bus.read(0x1001), 0xAA);
        assert_eq!(bus.read(0x1801), 0xAA);

        bus.write(0x1456, 0xBB);
        assert_eq!(bus.read(0x0456), 0xBB);
        assert_eq!(bus.read(0x0C56), 0xBB);
        assert_eq!(bus.read(0x1456), 0xBB);
        assert_eq!(bus.read(0x1C56), 0xBB);
    }

    #[test]
    fn plug_offset() {
        let mut bus = Bus::default();

        bus.plug(Box::new(Ram::<0x800>::new_zeroed(0x2000)), 0x200)
            .expect("failed to plug ram");

        bus.write(0x01FF, 0xAA);
        // open bus behaviour
        assert_eq!(bus.read(0x01FF), 0x00);

        bus.write(0x0201, 0xAA);
        assert_eq!(bus.read(0x0201), 0xAA);
        bus.write(0x21FF, 0xBB);
        assert_eq!(bus.read(0x21FF), 0xBB);

        bus.write(0x2200, 0xCC);
        // open bus behaviour
        assert_eq!(bus.read(0x2200), 0x00);
    }
}
