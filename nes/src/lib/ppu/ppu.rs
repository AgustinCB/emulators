use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use video_ram::VideoRam;

pub struct Ppu {
    ram: Rc<RefCell<Ram>>,
    video_ram: VideoRam,
}

impl Ppu {
    pub fn new(ram: Rc<RefCell<Ram>>) -> Ppu {
        Ppu {
            ram,
            video_ram: VideoRam::new(),
        }
    }
}