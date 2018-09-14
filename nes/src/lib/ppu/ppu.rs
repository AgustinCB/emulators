use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use ppu::register_2000::{Register2000, Register2000Connector};
use ppu::register_2001::{Register2001, Register2001Connector};
use ppu::register_2002::{Register2002, Register2002Connector};
use video_ram::VideoRam;

pub struct Ppu {
    ram: Rc<RefCell<Ram>>,
    register2000: Rc<RefCell<Register2000>>,
    register2001: Rc<RefCell<Register2001>>,
    register2002: Rc<RefCell<Register2002>>,
    video_ram: VideoRam,
}

impl Ppu {
    pub fn new(ram: Rc<RefCell<Ram>>) -> Ppu {
        let register2000= Rc::new(RefCell::new(Register2000::new()));
        let register2001= Rc::new(RefCell::new(Register2001::new()));
        let register2002 = Rc::new(RefCell::new(Register2002::new()));
        Ppu::set_connectors(&ram, &register2000, &register2001, &register2002);
        Ppu {
            ram,
            register2000,
            register2001,
            register2002,
            video_ram: VideoRam::new(),
        }
    }
    #[inline]
    fn set_connectors(
        ram: &Rc<RefCell<Ram>>,
        register2000: &Rc<RefCell<Register2000>>,
        register2001: &Rc<RefCell<Register2001>>,
        register2002: &Rc<RefCell<Register2002>>) {
        let mut m = ram.borrow_mut();
        m.io_registers[0].device =
            Some(Box::new(Register2000Connector::new(register2000)));
        m.io_registers[1].device =
            Some(Box::new(Register2001Connector::new(register2001)));
        m.io_registers[2].device =
            Some(Box::new(Register2002Connector::new(register2002)));
    }
}