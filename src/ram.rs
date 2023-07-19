pub struct Ram<const S: usize> {
    mem: [u8; S],
}

impl<const S: usize> Default for Ram<S> {
    fn default() -> Self {
        Self { mem: [0; S] }
    }
}

impl<const S: usize> Ram<S> {
    pub fn new(mem: [u8; S]) -> Self {
        Self { mem }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data;
    }
}
