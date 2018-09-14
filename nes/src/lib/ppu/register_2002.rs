use nes::InputOutputDevice;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register2002 {
    value: u8
}

/**
 * See page 34 of https://nesdev.com/NESDoc.pdf
 */
impl Register2002 {
    pub(crate) fn new() -> Register2002 {
        Register2002 {
            value: 0
        }
    }
    #[inline]
    pub(crate) fn enable_vram_writes(&mut self) {
        self.value = self.value & 0x10;
    }
    #[inline]
    pub(crate) fn disable_vram_writes(&mut self) {
        self.value = self.value & !0x10;
    }
    #[inline]
    pub(crate) fn is_scanline_sprite_count_bigger_than_eight(&self) -> bool {
        (self.value & 0x20) > 0
    }
    #[inline]
    pub(crate) fn is_there_pixel_overlap(&self) -> bool {
        (self.value & 0x40) > 0
    }
    #[inline]
    pub(crate) fn set_vblank_is_occurring(&mut self) {
        self.value = self.value | 0x80;
    }
    #[inline]
    pub(crate) fn set_vblank_stopped(&mut self) {
        self.value = self.value & !0x80;
    }
    #[inline]
    pub(crate) fn value(&self) -> u8 {
        self.value
    }
}

pub(crate) struct Register2002Connector {
    register: Rc<RefCell<Register2002>>,
}

impl Register2002Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2002>>) -> Register2002Connector {
        Register2002Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2002Connector {
    #[inline]
    fn read(&self) -> u8 {
        (*self.register.borrow()).value()
    }
    #[inline]
    fn write(&mut self, value: u8) -> u8 {
        value
    }
}