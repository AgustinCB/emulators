use nes::InputOutputDevice;
use ppu::SpriteMode;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register2000 {
    pub(crate) value: u8
}

/**
 * See page 17 of https://nesdev.com/NESDoc.pdf
 */
impl Register2000 {
    pub(crate) fn new() -> Register2000 {
        Register2000 {
            value: 0
        }
    }
    #[inline]
    pub(crate) fn is_nmi_enabled(&self) -> bool {
        (self.value & 0x80) > 0
    }
    #[inline]
    pub(crate) fn get_sprint_mode(&self) -> SpriteMode {
        if (self.value & 0x20) > 0 {
            SpriteMode::EightSixteen
        } else {
            SpriteMode::EightEight
        }
    }
    #[inline]
    pub(crate) fn get_memory_read_offset(&self) -> u8 {
        if (self.value & 0x04) > 0 {
            32 // vertical
        } else {
            1 // horizontal
        }
    }
}

pub(crate) struct Register2000Connector {
    register: Rc<RefCell<Register2000>>,
}

impl Register2000Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2000>>) -> Register2000Connector {
        Register2000Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2000Connector {
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