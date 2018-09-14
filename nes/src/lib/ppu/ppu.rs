use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use ppu::address_register::{AddressRegister, AddressRegisterConnector};
use ppu::register_2000::{Register2000, Register2000Connector};
use ppu::register_2001::{Register2001, Register2001Connector};
use ppu::register_2002::{Register2002, Register2002Connector};
use ppu::register_2007::{Register2007, Register2007Connector};
use ppu::register_4014::{Register4014, Register4014Connector};
use video_ram::VideoRam;

pub struct Ppu {
    ram: Rc<RefCell<Ram>>,
    register2000: Rc<RefCell<Register2000>>,
    register2001: Rc<RefCell<Register2001>>,
    register2002: Rc<RefCell<Register2002>>,
    register2003: Rc<RefCell<AddressRegister>>,
    register2005: Rc<RefCell<AddressRegister>>,
    register2006: Rc<RefCell<AddressRegister>>,
    register2007: Rc<RefCell<Register2007>>,
    register4014: Rc<RefCell<Register4014>>,
    video_ram: Rc<RefCell<VideoRam>>,
}

impl Ppu {
    pub fn new(ram: Rc<RefCell<Ram>>) -> Ppu {
        let video_ram = Rc::new(RefCell::new(VideoRam::new()));
        let register2000= Rc::new(RefCell::new(Register2000::new()));
        let register2001= Rc::new(RefCell::new(Register2001::new()));
        let register2002= Rc::new(RefCell::new(Register2002::new()));
        let register2003= Rc::new(RefCell::new(AddressRegister::new()));
        let register2005= Rc::new(RefCell::new(AddressRegister::new()));
        let register2006= Rc::new(RefCell::new(AddressRegister::new()));
        let register2007= Rc::new(RefCell::new(
            Register2007::new(&register2005, &register2006, &video_ram)));
        let register4014= Rc::new(RefCell::new(Register4014::new(&ram)));
        Ppu::set_connectors(&ram, &register2000, &register2001, &register2002, &register2003,
                            &register2005, &register2006, &register2007, &register4014);
        Ppu {
            ram,
            register2000,
            register2001,
            register2002,
            register2003,
            register2005,
            register2006,
            register2007,
            register4014,
            video_ram,
        }
    }
    #[inline]
    fn set_connectors(
        ram: &Rc<RefCell<Ram>>,
        register2000: &Rc<RefCell<Register2000>>,
        register2001: &Rc<RefCell<Register2001>>,
        register2002: &Rc<RefCell<Register2002>>,
        register2003: &Rc<RefCell<AddressRegister>>,
        register2005: &Rc<RefCell<AddressRegister>>,
        register2006: &Rc<RefCell<AddressRegister>>,
        register2007: &Rc<RefCell<Register2007>>,
        register4014: &Rc<RefCell<Register4014>>) {
        let mut m = ram.borrow_mut();
        m.io_registers[0].device =
            Some(Box::new(Register2000Connector::new(register2000)));
        m.io_registers[1].device =
            Some(Box::new(Register2001Connector::new(register2001)));
        m.io_registers[2].device =
            Some(Box::new(Register2002Connector::new(register2002)));
        m.io_registers[3].device =
            Some(Box::new(AddressRegisterConnector::new(register2003)));
        m.io_registers[5].device =
            Some(Box::new(AddressRegisterConnector::new(register2005)));
        m.io_registers[6].device =
            Some(Box::new(AddressRegisterConnector::new(register2006)));
        m.io_registers[7].device =
            Some(Box::new(Register2007Connector::new(register2007)));
        m.io_registers[28].device =
            Some(Box::new(Register4014Connector::new(register4014)));
    }
}