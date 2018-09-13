use ram::Ram;
use std::cell::RefCell;
use std::rc::Rc;
use video_ram::VideoRam;

pub(crate) struct Ppu {
    ram: Rc<RefCell<Ram>>,
    video_ram: VideoRam,
}

impl Ppu {
    pub(crate) fn new(ram: Rc<RefCell<Ram>>) -> Ppu {
        Ppu {
            ram,
            video_ram: VideoRam::new(),
        }
    }
}