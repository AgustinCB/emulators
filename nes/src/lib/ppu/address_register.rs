use nes::InputOutputDevice;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct AddressRegister {
    pub(crate) value: u8,
}

/**
 * See page 34 asnd 35 of https://nesdev.com/NESDoc.pdf
 * This is for registers 2003, 2005 and 2006.
 */
impl AddressRegister {
    pub(crate) fn new() -> AddressRegister {
        AddressRegister { value: 0 }
    }
}

pub(crate) struct AddressRegisterConnector {
    register: Rc<RefCell<AddressRegister>>,
}

impl AddressRegisterConnector {
    pub(crate) fn new(register: &Rc<RefCell<AddressRegister>>) -> AddressRegisterConnector {
        AddressRegisterConnector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for AddressRegisterConnector {
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
