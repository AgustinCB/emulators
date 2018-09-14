use nes::InputOutputDevice;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct Register2003 {
    pub(crate) value: u8
}

/**
 * See page 34 of https://nesdev.com/NESDoc.pdf
 */
impl Register2003 {
    pub(crate) fn new() -> Register2003 {
        Register2003 {
            value: 0
        }
    }
}

pub(crate) struct Register2003Connector {
    register: Rc<RefCell<Register2003>>,
}

impl Register2003Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2003>>) -> Register2003Connector {
        Register2003Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2003Connector {
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