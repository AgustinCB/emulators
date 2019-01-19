use nes::InputOutputDevice;
use ppu::address_register::AddressRegister;
use ppu::SpriteMemory;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register2004 {
    register2003: Rc<RefCell<AddressRegister>>,
    sprite_memory: Rc<RefCell<SpriteMemory>>,
}

/**
 * See page 34 of https://nesdev.com/NESDoc.pdf
 */
impl Register2004 {
    pub(crate) fn new(
        register2003: &Rc<RefCell<AddressRegister>>,
        sprite_memory: &Rc<RefCell<SpriteMemory>>,
    ) -> Register2004 {
        Register2004 {
            register2003: register2003.clone(),
            sprite_memory: sprite_memory.clone(),
        }
    }
    #[inline]
    fn get_address(&self) -> u8 {
        (*self.register2003.borrow()).value
    }
}

pub(crate) struct Register2004Connector {
    register: Rc<RefCell<Register2004>>,
}

impl Register2004Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2004>>) -> Register2004Connector {
        Register2004Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2004Connector {
    #[inline]
    fn read(&self) -> u8 {
        let address = self.register.borrow().get_address();
        let register = self.register.borrow();
        let sprite_memory = register.sprite_memory.borrow();
        sprite_memory[address as usize]
    }
    #[inline]
    fn write(&mut self, value: u8) -> u8 {
        let address = self.register.borrow().get_address();
        let register = self.register.borrow();
        (*register.sprite_memory.borrow_mut())[address as usize] = value;
        value
    }
}
