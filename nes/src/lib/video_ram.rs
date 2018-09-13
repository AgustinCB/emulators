pub(crate) struct VideoRam {
    pattern_tables: [u8; 0x2000],
    name_tables: [u8; 0x1000],
    palettes: [u8; 0x20],
}

impl VideoRam {
    pub(crate) fn new() -> VideoRam {
        VideoRam {
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
    use video_ram::VideoRam;

    #[test]
    fn it_should_get_from_pattern_tables() {
        let mut video_ram = VideoRam::new();
        video_ram.pattern_tables[0] = 0x42;
        assert_eq!(video_ram.get(0), 0x42);
    }

    #[test]
    fn it_should_get_from_name_tables() {
        let mut video_ram = VideoRam::new();
        video_ram.name_tables[0] = 0x42;
        assert_eq!(video_ram.get(0x2000), 0x42);
    }

    #[test]
    fn it_should_get_from_name_tables_with_mirroring() {
        let mut video_ram = VideoRam::new();
        video_ram.name_tables[0] = 0x42;
        assert_eq!(video_ram.get(0x3000), 0x42);
    }

    #[test]
    fn it_should_get_from_palettes() {
        let mut video_ram = VideoRam::new();
        video_ram.palettes[0] = 0x42;
        assert_eq!(video_ram.get(0x3F00), 0x42);
    }

    #[test]
    fn it_should_get_from_palettes_with_mirroring() {
        let mut video_ram = VideoRam::new();
        video_ram.palettes[0] = 0x42;
        assert_eq!(video_ram.get(0x3F20), 0x42);
    }

    #[test]
    fn it_should_set_in_pattern_tables() {
        let mut video_ram = VideoRam::new();
        video_ram.set(0, 0x42);
        assert_eq!(video_ram.pattern_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_name_tables() {
        let mut video_ram = VideoRam::new();
        video_ram.set(0x2000, 0x42);
        assert_eq!(video_ram.name_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_name_tables_with_mirroring() {
        let mut video_ram = VideoRam::new();
        video_ram.set(0x3000, 0x42);
        assert_eq!(video_ram.name_tables[0], 0x42);
    }

    #[test]
    fn it_should_set_in_palettes() {
        let mut video_ram = VideoRam::new();
        video_ram.set(0x3F00, 0x42);
        assert_eq!(video_ram.palettes[0], 0x42);
    }

    #[test]
    fn it_should_set_in_palettes_with_mirroring() {
        let mut video_ram = VideoRam::new();
        video_ram.set(0x3F20, 0x42);
        assert_eq!(video_ram.palettes[0], 0x42);
    }
}