use super::Mapper;

pub struct NRom128 {
    prg_rom: [u8; 0x4000],
    chr_rom: [u8; 0x2000],
}

impl Default for NRom128 {
    fn default() -> Self {
        Self {
            prg_rom: [0; 0x4000],
            chr_rom: [0; 0x2000],
        }
    }
}

impl Mapper for NRom128 {
    fn prg_read(&self, addr: u16) -> Option<u8> {
        self.prg_rom
            .get(addr.wrapping_sub(0x8000) as usize)
            .copied()
    }

    fn prg_write(&mut self, _addr: u16, _data: u8) {
        // since this is rom I don't believe you can write on it
        // self.prg_rom[addr as usize] = data;
    }

    fn chr_read(&self, addr: u16) -> Option<u8> {
        self.prg_rom.get(addr as usize).copied()
    }

    fn chr_write(&mut self, addr: u16, data: u8) {
        self.chr_rom[addr as usize] = data;
    }
}

pub struct NRom256 {
    prg_rom: [u8; 0x8000],
    chr_rom: [u8; 0x2000],
}

impl Default for NRom256 {
    fn default() -> Self {
        Self {
            prg_rom: [0; 0x8000],
            chr_rom: [0; 0x2000],
        }
    }
}

impl Mapper for NRom256 {
    fn prg_read(&self, addr: u16) -> Option<u8> {
        self.prg_rom
            .get(addr.wrapping_sub(0x8000) as usize)
            .copied()
    }

    fn prg_write(&mut self, _addr: u16, _data: u8) {
        // since this is rom I don't believe you can write on it
        // self.prg_data[addr as usize] = data;
    }

    fn chr_read(&self, addr: u16) -> Option<u8> {
        self.prg_rom.get(addr as usize).copied()
    }

    fn chr_write(&mut self, addr: u16, data: u8) {
        self.chr_rom[addr as usize] = data;
    }
}
