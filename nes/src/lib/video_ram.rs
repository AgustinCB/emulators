use ram::{IORegister, Ram};
use std::cell::RefCell;
use std::rc::Rc;

type VideoRegister = Rc<RefCell<IORegister>>;

struct VideoRam {
    register2000: VideoRegister,
    register2001: VideoRegister,
    register2002: VideoRegister,
    register2003: VideoRegister,
    register2004: VideoRegister,
    register2005: VideoRegister,
    register2006: VideoRegister,
    register2007: VideoRegister,
    register4014: VideoRegister,
    pattern_tables: [u8; 0x2000],
    name_tables: [u8; 0x1000],
    palettes: [u8; 0x20],
}

impl VideoRam {
    pub(crate) fn new(ram: Rc<Ram>) -> VideoRam {
        VideoRam {
            register2000: ram.io_registers[0].clone(),
            register2001: ram.io_registers[1].clone(),
            register2002: ram.io_registers[2].clone(),
            register2003: ram.io_registers[3].clone(),
            register2004: ram.io_registers[4].clone(),
            register2005: ram.io_registers[5].clone(),
            register2006: ram.io_registers[6].clone(),
            register2007: ram.io_registers[7].clone(),
            register4014: ram.io_registers[28].clone(),
            pattern_tables: [0; 0x2000],
            name_tables: [0; 0x1000],
            palettes: [0; 0x20],
        }
    }

    pub(crate) fn get(&self, index: u16) -> u8 {
        if index < 0x2000 {
            self.pattern_tables[index as usize]
        } else if index < 0x3000 {
            self.name_tables[index as usize - 0x2000]
        } else if index < 0x3F00 {
            self.name_tables[index as usize - 0x3000]
        } else if index < 0x3F20 {
            self.palettes[index as usize - 0x3F00]
        } else if index < 0x4000 {
            self.palettes[(index as usize - 0x3F20) % 0x20]
        } else {
            self.get(index % 0x4000)
        }
    }

    pub(crate) fn set(&mut self, index: u16, new_value: u8) {
        if index < 0x2000 {
            self.pattern_tables[index as usize] = new_value;
        } else if index < 0x3000 {
            self.name_tables[index as usize - 0x2000] = new_value;
        } else if index < 0x3F00 {
            self.name_tables[index as usize - 0x3000] = new_value;
        } else if index < 0x3F20 {
            self.palettes[index as usize - 0x3F00] = new_value;
        } else if index < 0x4000 {
            self.palettes[(index as usize - 0x3F20) % 0x20] = new_value;
        } else {
            self.set(index % 0x4000, new_value)
        }
    }
}

#[cfg(test)]
mod tests {
    use mos6502cpu::Memory;
    use ram::{Ram, ROM_SIZE};
    use std::rc::Rc;
    use video_ram::VideoRam;

    # [test]
    fn it_should_map_to_the_correct_bits_in_ram() {
        let mut m = Ram::new([0; ROM_SIZE]);
        m.set(0x2000, 0x42);
        m.set(0x2001, 0x43);
        m.set(0x2002, 0x44);
        m.set(0x2003, 0x45);
        m.set(0x2004, 0x46);
        m.set(0x2005, 0x47);
        m.set(0x2006, 0x48);
        m.set(0x2007, 0x49);
        m.set(0x4014, 0x41);
        let video_ram = VideoRam::new(Rc::new(m));
        assert_eq!(video_ram.register2000.borrow().current, 0x42);
        assert_eq!(video_ram.register2001.borrow().current, 0x43);
        assert_eq!(video_ram.register2002.borrow().current, 0x44);
        assert_eq!(video_ram.register2003.borrow().current, 0x45);
        assert_eq!(video_ram.register2004.borrow().current, 0x46);
        assert_eq!(video_ram.register2005.borrow().current, 0x47);
        assert_eq!(video_ram.register2006.borrow().current, 0x48);
        assert_eq!(video_ram.register4014.borrow().current, 0x41);
    }

    #[test]
    fn it_should_get_from_pattern_tables() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.pattern_tables[0] = 0x42;
        assert_eq!(video_ram.get(0), 0x42);
    }

    #[test]
    fn it_should_get_from_name_tables() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.name_tables[0] = 0x42;
        assert_eq!(video_ram.get(0x2000), 0x42);
    }

    #[test]
    fn it_should_get_from_name_tables_with_mirroring() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.name_tables[0] = 0x42;
        assert_eq!(video_ram.get(0x3000), 0x42);
    }

    #[test]
    fn it_should_get_from_palettes() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.palettes[0] = 0x42;
        assert_eq!(video_ram.get(0x3F00), 0x42);
    }

    #[test]
    fn it_should_get_from_palettes_with_mirroring() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.palettes[0] = 0x42;
        assert_eq!(video_ram.get(0x3F20), 0x42);
    }

    #[test]
    fn it_should_set_in_pattern_tables() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.set(0, 0x42);
        assert_eq!(video_ram.pattern_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_name_tables() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.set(0x2000, 0x42);
        assert_eq!(video_ram.name_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_name_tables_with_mirroring() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.set(0x3000, 0x42);
        assert_eq!(video_ram.name_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_palettes() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.set(0x3F00, 0x42);
        assert_eq!(video_ram.palettes[0], 0x42);
    }

    #[test]
    fn it_should_set_in_palettes_with_mirroring() {
        let m = Ram::new([0; ROM_SIZE]);
        let mut video_ram = VideoRam::new(Rc::new(m));
        video_ram.set(0x3F20, 0x42);
        assert_eq!(video_ram.palettes[0], 0x42);
    }
}