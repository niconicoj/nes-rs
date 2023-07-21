use std::{cell::RefCell, rc::Rc};

use crate::ram::Ram;

#[derive(Clone)]
pub struct Bus {
    data: Rc<RefCell<[u8; 0x10000]>>,
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new([0; 0x10000])),
        }
    }
}

impl Bus {
    pub fn read(&self, addr: u16) -> u8 {
        self.data.borrow()[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.data.borrow_mut()[addr as usize] = data;
    }
}
