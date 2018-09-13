use ppu::Ppu;
use ram::{Ram, ROM_SIZE};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) trait InputOutputDevice {
    fn read(&self) -> u8;
    fn write(&mut self, value: u8) -> u8;
}

pub struct Nes {
    pub ram: Rc<RefCell<Ram>>,
    ppu: Ppu,
}

impl Nes {
    pub fn new(rom: [u8; ROM_SIZE]) -> Nes {
        let ram = Rc::new(RefCell::new(Ram::new(rom)));
        Nes {
            ppu: Ppu::new(ram.clone()),
            ram,
        }
    }
}