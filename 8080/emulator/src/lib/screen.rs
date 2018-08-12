use std::cell::Cell;

const FRAME_BUFFER_START_ADDRESS: usize = 0x2400;
const COLUMN_LIMIT_BETWEEN_INTERRUPTIONS: u16 = 96;
pub(crate) const SCREEN_WIDTH: usize = 224;
pub(crate) const SCREEN_HEIGHT: usize = 256;

pub(crate) type Pixel = bool;
pub(crate) type Line = [Pixel; SCREEN_WIDTH];
pub(crate) type ScreenLayout = [Line; SCREEN_HEIGHT];

pub(crate) trait Screen {
    fn on_mid_screen(&mut self);
    fn on_full_screen(&mut self);
    fn get_pixels(&self) -> &ScreenLayout;
}

fn get_bits(byte: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    let mut mask: u8 = 0x01;
    let mut i = 0;
    while mask < 0x80 {
        bits[i] = (byte & mask) > 0;
        mask <<= 1;
        i += 1;
    }
    bits[i] = (byte & mask) > 0;
    return bits;
}

pub(crate) struct TermScreen {
    memory: Vec<Cell<u8>>,
    lines: ScreenLayout,
}

impl TermScreen {
    pub(crate) fn new(memory: &Vec<Cell<u8>>) -> TermScreen {
        let lines = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        TermScreen {
            lines,
            memory: memory.iter().map(|c| Cell::clone(c)).collect(),
        }
    }

    fn update_columns(&mut self, start_column: u16, end_column: u16) {
        for column in start_column..end_column {
            let start_address = FRAME_BUFFER_START_ADDRESS as u16 + column * 0x20;
            let end_address = start_address + 0x20;
            for start_line_address in start_address..end_address {
                let start_line = start_line_address - start_address;
                let bits = get_bits(self.memory[start_line_address as usize].get());
                let mut counter: u16 = 0;
                for bit in bits.iter() {
                    self.lines[(start_line+counter) as usize][column as usize] = *bit;
                    counter += 1;
                }
            }
        }
    }
}

impl Screen for TermScreen {
    fn on_mid_screen(&mut self) {
        self.update_columns(COLUMN_LIMIT_BETWEEN_INTERRUPTIONS, SCREEN_WIDTH as u16);
    }

    fn on_full_screen(&mut self) {
        self.update_columns(0, COLUMN_LIMIT_BETWEEN_INTERRUPTIONS);
    }

    fn get_pixels(&self) -> &ScreenLayout {
        &(self.lines)
    }
}