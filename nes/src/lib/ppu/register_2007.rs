use ppu::address_register::AddressRegister;
use nes::InputOutputDevice;
use std::cell::RefCell;
use std::rc::Rc;
use video_ram::VideoRam;

#[inline]
fn two_bytes_to_word(high_byte: u8, low_byte: u8) -> u16 {
    (high_byte as u16) << 8 | (low_byte as u16)
}


pub(crate) struct Register2007 {
    register2005: Rc<RefCell<AddressRegister>>,
    register2006: Rc<RefCell<AddressRegister>>,
    video_ram: Rc<RefCell<VideoRam>>,
}

/**
 * See page 35 of https://nesdev.com/NESDoc.pdf
 */
impl Register2007 {
    pub(crate) fn new(
        register2005: &Rc<RefCell<AddressRegister>>,
        register2006: &Rc<RefCell<AddressRegister>>,
        video_ram: &Rc<RefCell<VideoRam>>) -> Register2007 {
        Register2007 {
            register2005: register2005.clone(),
            register2006: register2006.clone(),
            video_ram: video_ram.clone()
        }
    }
    #[inline]
    fn get_address(&self) -> u16 {
        let lb = (*self.register2005.borrow()).value;
        let hb = (*self.register2006.borrow()).value;
        two_bytes_to_word(hb, lb)
    }
}

pub(crate) struct Register2007Connector {
    register: Rc<RefCell<Register2007>>,
}

impl Register2007Connector {
    pub(crate) fn new(register: &Rc<RefCell<Register2007>>) -> Register2007Connector {
        Register2007Connector {
            register: register.clone(),
        }
    }
}

impl InputOutputDevice for Register2007Connector {
    #[inline]
    fn read(&self) -> u8 {
        let address = self.register.borrow().get_address();
        let register = self.register.borrow();
        let vram = register.video_ram.borrow();
        vram.get(address)
    }
    #[inline]
    fn write(&mut self, value: u8) -> u8 {
        let address = self.register.borrow().get_address();
        let register = self.register.borrow();
        register.video_ram.borrow_mut().set(address, value);
        value
    }
}