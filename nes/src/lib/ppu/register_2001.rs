use nes::InputOutputDevice;
use ppu::SpriteMode;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register2001 {
    pub(crate) value: u8
}

/**
 * See page 18 of https://nesdev.com/NESDoc.pdf
 */
impl Register2001 {
    pub(crate) fn new() -> Register2001 {
        Register2001 {
            value: 0
        }
    }
    #[inline]
    pub(crate) fn is_background_shown(&self) -> bool {
        (self.value & 0x08) > 0
    }
    #[inline]
    pub(crate) fn are_sprites_shown(&self) -> bool {
        (self.value & 0x10) > 0
    }
}

pub(crate) struct Register2001Connector {
    register: Rc<RefCell<Register2001>>,
}

impl Register2001Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2001>>) -> Register2001Connector {
        Register2001Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2001Connector {
    #[inline]
    fn read(&self) -> u8 {
        (*self.register.borrow()).value
    }
    #[inline]
    fn write(&mut self, value: u8) -> u8 {
        (*self.register.borrow_mut()).value = value;
        value
    }
}