use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use ppu::register_2000::{Register2000, Register2000Connector};
use video_ram::VideoRam;

pub struct Ppu {
    ram: Rc<RefCell<Ram>>,
    register2000: Rc<RefCell<Register2000>>,
    video_ram: VideoRam,
}

impl Ppu {
    pub fn new(ram: Rc<RefCell<Ram>>) -> Ppu {
        let register2000 = Rc::new(RefCell::new(Register2000::new()));
        {
            let mut m = ram.borrow_mut();
            m.io_registers[0].device =
                Some(Box::new(Register2000Connector::new(&register2000)));
        }
        Ppu {
            ram,
            register2000,
            video_ram: VideoRam::new(),
        }
    }
}