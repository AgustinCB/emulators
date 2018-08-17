extern crate ears;

use self::ears::{AudioController, Sound};
use super::super::ConsoleError;
use super::super::cpu::OutputDevice;
use super::super::failure::Error;

pub struct SoundPort1 {
    last_value: u8,
    background: Sound,
    instant_sound_1: Sound,
    instant_sound_2: Sound,
    instant_sound_3: Sound,
}

pub struct SoundPort2 {
    last_value: u8,
    instant_sound_4: Sound,
    instant_sound_5: Sound,
    instant_sound_6: Sound,
    instant_sound_7: Sound,
    instant_sound_8: Sound,
}

fn create_sound(path: &str) -> Result<Sound, Error> {
    Sound::new(path).map_err(|e| Error::from(ConsoleError::CantCreateSound { msg: e }))
}

impl SoundPort1 {
    pub fn new(folder: &str) -> Result<SoundPort1, Error> {
        Ok(SoundPort1 {
            last_value: 0,
            background: {
                let mut sound = create_sound(&format!("{}/0.wav", folder))?;
                sound.set_looping(true);
                sound
            },
            instant_sound_1: create_sound(&format!("{}/1.wav", folder))?,
            instant_sound_2: create_sound(&format!("{}/2.wav", folder))?,
            instant_sound_3: create_sound(&format!("{}/3.wav", folder))?,
        })
    }
}

impl SoundPort2 {
    pub fn new(folder: &str) -> Result<SoundPort2, Error> {
        Ok(SoundPort2 {
            last_value: 0,
            instant_sound_4: create_sound(&format!("{}/4.wav", folder))?,
            instant_sound_5: create_sound(&format!("{}/5.wav", folder))?,
            instant_sound_6: create_sound(&format!("{}/6.wav", folder))?,
            instant_sound_7: create_sound(&format!("{}/7.wav", folder))?,
            instant_sound_8: create_sound(&format!("{}/8.wav", folder))?,
        })
    }
}

macro_rules! maybe_play_instant_sound {
    ($position:expr, $byte:ident, $this:ident, $sound:ident) => {
        if ($byte & $position) ^ ($byte & $this.last_value) > 0 &&
            !$this.$sound.is_playing () {
            $this.$sound.play();
        }
    }
}
impl OutputDevice for SoundPort1 {
    fn write(&mut self, byte: u8) {
        if (byte & 0x01) ^ (byte & self.last_value) > 0 {
            if self.background.is_playing() {
                self.background.stop();
            } else {
                self.background.play();
            }
        }
        maybe_play_instant_sound!(0x02, byte, self, instant_sound_1);
        maybe_play_instant_sound!(0x04, byte, self, instant_sound_2);
        maybe_play_instant_sound!(0x08, byte, self, instant_sound_3);
    }
}

impl OutputDevice for SoundPort2 {
    fn write(&mut self, byte: u8) {
        maybe_play_instant_sound!(0x01, byte, self, instant_sound_4);
        maybe_play_instant_sound!(0x02, byte, self, instant_sound_5);
        maybe_play_instant_sound!(0x04, byte, self, instant_sound_6);
        maybe_play_instant_sound!(0x08, byte, self, instant_sound_7);
        maybe_play_instant_sound!(0x10, byte, self, instant_sound_8);
    }
}