use ram::{Ram, ROM_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use video_ram::VideoRam;

pub struct Nes {
    pub ram: Rc<RefCell<Ram>>,
    video_ram: VideoRam,
}

impl Nes {
    pub fn new(rom: [u8; ROM_SIZE]) -> Nes {
        let ram = Rc::new(RefCell::new(Ram::new(rom)));
        Nes {
            video_ram: VideoRam::new(ram.clone()),
            ram,
        }
    }
}