pub struct Ram<const S: usize> {
    data: [u8; S],
}

impl<const S: usize> Default for Ram<S> {
    fn default() -> Self {
        Self { data: [0; S] }
    }
}

impl<const S: usize> Ram<S> {
    pub fn read(&self, addr: u16) -> u8 {
        // modulo = mirroring addresses down to the real address
        self.data[((addr as usize) % S) as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.data[((addr as usize) % S) as usize] = data;
    }
}

#[cfg(test)]
mod tests {
    use crate::ram::Ram;

    #[test]
    fn ram_mirroring() {
        // make 2k of ram mirrored on an 8k space
        let mut ram = Ram::<0x800>::default();

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
