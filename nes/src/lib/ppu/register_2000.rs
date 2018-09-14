use nes::InputOutputDevice;
use ppu::{PpuMode, SpriteMode};
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
    pub(crate) fn get_name_table(&self) -> u8 {
        self.value & 0x03
    }
    #[inline]
    pub(crate) fn get_memory_read_offset(&self) -> u8 {
        if (self.value & 0x04) > 0 {
            32 // vertical
        } else {
            1 // horizontal
        }
    }
    #[inline]
    pub(crate) fn get_sprite_pattern_table(&self) -> u8 {
        (self.value & 0x08) >> 3
    }
    #[inline]
    pub(crate) fn get_background_pattern_table(&self) -> u8 {
        (self.value & 0x10) >> 4
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
    pub(crate) fn get_ppu_mode(&self) -> PpuMode {
        if (self.value & 0x40) > 0 {
            PpuMode::Master
        } else {
            PpuMode::Slave
        }
    }
    #[inline]
    pub(crate) fn is_nmi_enabled(&self) -> bool {
        (self.value & 0x80) > 0
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