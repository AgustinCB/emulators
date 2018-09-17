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

    pub(crate) fn get_tile(&self, index: u16) -> Vec<Vec<u8>> {
        let from = index.wrapping_mul(0x10) as usize;
        let to = index.wrapping_add(1).wrapping_mul(0x10) as usize;
        let mut result = Vec::with_capacity(8);
        for i in from..(to/2) {
            let current_row = self.get_tile_row(i);
            result.push(current_row);
        }
        result
    }

    fn get_tile_row(&self, i: usize) -> Vec<u8> {
        let mut current_row = Vec::with_capacity(8);
        let first_byte = self.pattern_tables[i];
        let second_byte = self.pattern_tables[i + 8];
        for j in 0..8 {
            let first_bit = ((first_byte & 2_u8.pow(j)) > 0) as u8;
            let second_bit = ((second_byte & 2_u8.pow(j)) > 0) as u8;
            current_row.push(first_bit | (second_bit << 1));
        }
        current_row.reverse();
        current_row
    }
}

#[cfg(test)]
mod tests {
    use ppu::video_ram::VideoRam;

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

    #[test]
    fn it_should_correctly_get_a_tile_from_the_patterns_table() {
        let mut video_ram = VideoRam::new();
        video_ram.pattern_tables[0x0] = 0x10;
        video_ram.pattern_tables[0x1] = 0x00;
        video_ram.pattern_tables[0x2] = 0x44;
        video_ram.pattern_tables[0x3] = 0x00;
        video_ram.pattern_tables[0x4] = 0xFE;
        video_ram.pattern_tables[0x5] = 0x00;
        video_ram.pattern_tables[0x6] = 0x82;
        video_ram.pattern_tables[0x7] = 0x00;
        video_ram.pattern_tables[0x8] = 0x00;
        video_ram.pattern_tables[0x9] = 0x28;
        video_ram.pattern_tables[0xa] = 0x44;
        video_ram.pattern_tables[0xb] = 0x82;
        video_ram.pattern_tables[0xc] = 0x00;
        video_ram.pattern_tables[0xd] = 0x82;
        video_ram.pattern_tables[0xe] = 0x82;
        video_ram.pattern_tables[0xf] = 0x00;
        let expected_result: Vec<Vec<u8>> = vec![
            vec![0, 0, 0, 1, 0, 0, 0, 0],
            vec![0, 0, 2, 0, 2, 0, 0, 0],
            vec![0, 3, 0, 0, 0, 3, 0, 0],
            vec![2, 0, 0, 0, 0, 0, 2, 0],
            vec![1, 1, 1, 1, 1, 1, 1, 0],
            vec![2, 0, 0, 0, 0, 0, 2, 0],
            vec![3, 0, 0, 0, 0, 0, 3, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0],
        ];
        assert_eq!(expected_result, video_ram.get_tile(0));
    }
}