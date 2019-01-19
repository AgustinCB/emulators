const COLUMN_LIMIT_BETWEEN_INTERRUPTIONS: u16 = 96;
pub(crate) const SCREEN_WIDTH: usize = 224;
pub(crate) const SCREEN_HEIGHT: usize = 256;

pub(crate) type Pixel = bool;
pub(crate) type Line = [Pixel; SCREEN_WIDTH];
pub(crate) type ScreenLayout = [Line; SCREEN_HEIGHT];

pub(crate) trait Screen {
    fn on_mid_screen(&mut self, memory: &[u8]);
    fn on_full_screen(&mut self, memory: &[u8]);
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

pub(crate) struct GameScreen {
    lines: ScreenLayout,
}

impl GameScreen {
    pub(crate) fn new() -> GameScreen {
        let lines = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        GameScreen { lines }
    }

    fn update_columns(&mut self, start_column: u16, end_column: u16, frame_buffer: &[u8]) {
        for column in start_column..end_column {
            for line_group in 0..0x20 {
                let address = column * 0x20 + line_group;
                let bits = get_bits(frame_buffer[address as usize]);
                for line_index in 0..8 {
                    let line = SCREEN_HEIGHT - 1 - (line_group * 8 + line_index) as usize;
                    self.lines[line][column as usize] = bits[line_index as usize];
                }
            }
        }
    }
}

impl Screen for GameScreen {
    fn on_mid_screen(&mut self, frame_buffer: &[u8]) {
        self.update_columns(
            COLUMN_LIMIT_BETWEEN_INTERRUPTIONS,
            SCREEN_WIDTH as u16,
            frame_buffer,
        );
    }

    fn on_full_screen(&mut self, frame_buffer: &[u8]) {
        self.update_columns(0, COLUMN_LIMIT_BETWEEN_INTERRUPTIONS, frame_buffer);
    }

    fn get_pixels(&self) -> &ScreenLayout {
        &(self.lines)
    }
}

#[cfg(test)]
mod tests {
    use super::super::console::FRAME_BUFFER_SIZE;
    use super::{GameScreen, Screen, SCREEN_WIDTH};

    #[test]
    fn it_should_correctly_translate_from_memory() {
        let expected_output: Vec<Vec<bool>> = [
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [false; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
            [true; SCREEN_WIDTH],
        ]
        .iter()
        .map(|s| s.to_vec())
        .collect();
        let mut memory = [0; FRAME_BUFFER_SIZE];
        for counter in (0x1f..FRAME_BUFFER_SIZE).step_by(0x20) {
            memory[counter] = 0xf0;
        }
        for counter in (0x00..(FRAME_BUFFER_SIZE - 0x1f)).step_by(0x20) {
            memory[counter] = 0x0f;
        }
        let mut screen = GameScreen::new();
        screen.on_full_screen(&memory);
        screen.on_mid_screen(&memory);
        let actual_output: Vec<Vec<bool>> =
            screen.get_pixels().iter().map(|s| s.to_vec()).collect();
        assert_eq!(expected_output, actual_output);
    }
}
