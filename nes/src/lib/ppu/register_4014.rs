use mos6502cpu::Memory;
use nes::InputOutputDevice;
use ppu::SpriteMode;
use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register4014 {
    ram: Rc<RefCell<Ram>>,
    pub(crate) sprite_memory: [u8; 256],
    value: u8,
}

/**
 * See page 18 of https://nesdev.com/NESDoc.pdf, section direct memory access
 */
impl Register4014 {
    pub(crate) fn new(ram: &Rc<RefCell<Ram>>) -> Register4014 {
        Register4014 {
            ram: ram.clone(),
            sprite_memory: [0; 256],
            value: 0,
        }
    }
}

pub(crate) struct Register4014Connector {
    register: Rc<RefCell<Register4014>>,
}

impl Register4014Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register4014>>) -> Register4014Connector {
        Register4014Connector {
            register: register.clone(),
        }
    }
    fn save_to_sprite_memory(&mut self, starting_address: u16) {
        let mut register = self.register.borrow_mut();
        for i in 0..256 {
            let ram_index = starting_address.wrapping_add(i as u16);
            let value = register.ram.borrow().get(ram_index);
            register.sprite_memory[i] = value;
        }
    }
}

impl InputOutputDevice for Register4014Connector {
    #[inline]
    fn read(&self) -> u8 {
        (*self.register.borrow()).value
    }
    #[inline]
    fn write(&mut self, value: u8) -> u8 {
        (*self.register.borrow_mut()).value = value;
        self.save_to_sprite_memory((value as u16).wrapping_mul(0x100));
        value
    }
}