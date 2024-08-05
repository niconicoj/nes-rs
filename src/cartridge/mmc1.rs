use bevy::log::info;
use bevy_fundsp::prelude::Num;

use super::{
    mapper::{MapResult, Mapper},
    Mirroring,
};

pub struct Mmc1 {
    load_register: u8,
    load_count: u8,
    control_register: u8,
    vram: [u8; 0x2000],
    prg_bank_nb: usize,
    /// bank id for prg rom for all three mode
    /// in order :
    ///     0 -> first bank id (last bank is fixed)
    ///     1 -> last bank id (first bank is fixed)
    ///     2 -> full bank
    prg_bank_id: [usize; 3],
    chr_bank_nb: usize,
    chr_bank_id: [usize; 3],
}

impl Mmc1 {
    pub fn new(prg_bank_nb: usize, chr_bank_nb: usize) -> Self {
        Self {
            load_register: 0x00,
            load_count: 0x00,
            control_register: 0x1C,
            vram: [0; 0x2000],
            prg_bank_id: [0; 3],
            prg_bank_nb,
            chr_bank_id: [0; 3],
            chr_bank_nb,
        }
    }

    pub fn set_mirroring(&mut self, mirroring: Mirroring) {
        self.control_register &= 0xFC;
        self.control_register |= mirroring as u8;
    }
}

impl Mapper for Mmc1 {
    fn cpu_map_read(&self, addr: u16) -> Option<MapResult> {
        match (addr, (self.control_register >> 2) & 0x03) {
            (0x6000..=0x7FFF, _) => Some(MapResult::Instant {
                data: self.vram[(addr & 0x1FFF) as usize],
            }),
            // fixed bank
            (0x8000..=0xBFFF, 0x10) | (0x8000..=0xBFFF, 0x11) => Some(MapResult::Rom {
                bank: self.prg_bank_id[0],
                addr,
            }),
            (0xC000..=0xFFFF, 0x10) | (0xC000..=0xFFFF, 0x11) => Some(MapResult::Rom {
                bank: self.prg_bank_id[1],
                addr,
            }),
            // full bank
            (0x8000..=0xBFFF, _) => Some(MapResult::Rom {
                bank: self.prg_bank_id[2],
                addr,
            }),
            (0xC000..=0xFFFF, _) => Some(MapResult::Rom {
                bank: std::cmp::Ord::min(self.prg_bank_id[2] + 1, self.prg_bank_nb - 1),
                addr,
            }),
            _ => None,
        }
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> Option<MapResult> {
        info!("mmc1 write {:04X} {:02X}", addr, data);
        match addr {
            0x6000..=0x7FFF => {
                self.vram[(addr & 0x1FFF) as usize] = data;
                None
            }
            0x8000..=0xFFFF => {
                if data & 0x80 != 0 {
                    self.control_register = self.control_register | 0x0C;
                    self.load_register = 0;
                    self.load_count = 0;
                } else {
                    self.load_register = self.load_register >> 1 | ((data & 0x01) << 4);
                    self.load_count += 1;
                    if self.load_count == 5 {
                        let target = (addr >> 13) & 0x03;
                        match target {
                            0x00 => {
                                self.control_register = self.load_register & 0x1F;
                            }
                            0x01 => {
                                if self.control_register & 0x10 != 0 {
                                    self.chr_bank_id[0] = (self.load_register & 0x1F) as usize;
                                } else {
                                    self.chr_bank_id[2] = (self.load_register & 0x1E) as usize;
                                }
                            }
                            0x02 => {
                                if self.control_register & 0x10 != 0 {
                                    self.chr_bank_id[1] = (self.load_register & 0x1F) as usize;
                                }
                            }
                            0x03 => match (self.control_register >> 2) & 0x03 {
                                0x00 | 0x01 => {
                                    self.prg_bank_id[2] =
                                        ((self.load_register & 0x0E) >> 1) as usize;
                                }
                                0x02 => {
                                    self.prg_bank_id[0] = 0;
                                    self.prg_bank_id[1] = (self.load_register & 0x0F) as usize;
                                }
                                0x03 => {
                                    self.prg_bank_id[0] = (self.load_register & 0x0F) as usize;
                                    self.prg_bank_id[1] = self.prg_bank_nb - 1;
                                }
                                _ => {}
                            },
                            _ => {}
                        };
                        self.load_register = 0x00;
                        self.load_count = 0;
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn ppu_map_read(&self, addr: u16) -> Option<MapResult> {
        if addr < 0x2000 {
            if self.chr_bank_nb == 0 {
                return Some(MapResult::Rom { bank: 0, addr });
            } else {
                match (addr, (self.control_register >> 4) & 0x01) {
                    (0x0000..=0x0FFF, 0x01) => Some(MapResult::Rom {
                        bank: self.chr_bank_id[0],
                        addr,
                    }),
                    (0x1000..=0x1FFF, 0x01) => Some(MapResult::Rom {
                        bank: self.chr_bank_id[1],
                        addr,
                    }),
                    (0x0000..=0x2000, _) => Some(MapResult::Rom {
                        bank: self.chr_bank_id[2],
                        addr,
                    }),
                    _ => None,
                }
            }
        } else {
            None
        }
    }

    fn ppu_map_write(&self, _addr: u16, _data: u8) -> Option<MapResult> {
        None
    }

    fn mirroring(&self) -> Option<Mirroring> {
        Some(match self.control_register & 0x03 {
            0x00 => Mirroring::OneScreenLo,
            0x01 => Mirroring::OneScreenHi,
            0x02 => Mirroring::Vertical,
            0x03 => Mirroring::Horizontal,
            _ => unreachable!(),
        })
    }
}
