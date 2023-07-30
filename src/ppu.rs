#[derive(Default)]
pub struct Ppu {
    cycles: usize,
}

impl Ppu {
    pub fn tick(&mut self) {
        self.cycles += 1;
    }
}
