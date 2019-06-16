use mos6502cpu::Memory;
use nes::InputOutputDevice;
use ppu::SpriteMemory;
use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register4014 {
    ram: Rc<RefCell<Ram>>,
    pub(crate) sprite_memory: Rc<RefCell<SpriteMemory>>,
    value: u8,
}

/**
 * See page 18 of https://nesdev.com/NESDoc.pdf, section direct memory access
 */
impl Register4014 {
    pub(crate) fn new(
        ram: &Rc<RefCell<Ram>>,
        sprite_memory: &Rc<RefCell<SpriteMemory>>,
    ) -> Register4014 {
        Register4014 {
            ram: ram.clone(),
            sprite_memory: sprite_memory.clone(),
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
            (*register.sprite_memory.borrow_mut())[i] = value;
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
        // TODO: This should keep the CPU busy for 512 cycles.
        // I'm not really sure how to express that right now. Possible ideas:
        // 1. Let the user pass a possible delay to the execute_instruction method.
        // 2. Make the Memory trait somehow express the delays in reading to it.
        self.save_to_sprite_memory(u16::from(value).wrapping_mul(0x100));
        value
    }
}
