use crate::bus::BusDevice;

pub struct Ram<const S: usize> {
    data: [u8; S],
    /// this is useful for mirroring
    /// for example if you want to have 2k of memory mirrored over 8k addresses
    addr_space: usize,
}

impl<const S: usize> Default for Ram<S> {
    fn default() -> Self {
        Self {
            data: [0; S],
            addr_space: S,
        }
    }
}

impl<const S: usize> Ram<S> {
    pub fn new_zeroed(addr_space: usize) -> Self {
        Self {
            data: [0; S],
            addr_space,
        }
    }
    pub fn new(data: [u8; S], addr_space: usize) -> Self {
        Self { data, addr_space }
    }
}

impl<const S: usize> BusDevice for Ram<S> {
    fn addr_space(&self) -> usize {
        self.addr_space
    }

    fn read(&self, addr: u16) -> u8 {
        // modulo = mirroring addresses down to the real address
        self.data[((addr as usize) % S) as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.data[((addr as usize) % S) as usize] = data;
    }
}

mod tests {
    use crate::{bus::BusDevice, ram::Ram};

    #[test]
    fn ram_mirroring() {
        // make 2k of ram mirrored on an 8k space
        let mut ram = Ram::<0x800>::new_zeroed(0x2000);

        ram.write(0x0001, 0xAA);
        assert_eq!(ram.read(0x0001), 0xAA);
        assert_eq!(ram.read(0x0801), 0xAA);
        assert_eq!(ram.read(0x1001), 0xAA);
        assert_eq!(ram.read(0x1801), 0xAA);

        ram.write(0x1456, 0xBB);
        assert_eq!(ram.read(0x0456), 0xBB);
        assert_eq!(ram.read(0x0C56), 0xBB);
        assert_eq!(ram.read(0x1456), 0xBB);
        assert_eq!(ram.read(0x1C56), 0xBB);
    }
}
