extern crate rodio;

use std::io::BufReader;
use std::fs::File;
use self::rodio::{Sink, Source, Decoder, Device};
use super::super::failure::Error;
use super::super::ConsoleError;
use super::intel8080cpu::OutputDevice;

pub struct SoundPort1 {
    last_value: u8,
    device: Device,
    background: Sink,
    instant_sound_1: String,
    instant_sound_2: String,
    instant_sound_3: String,
    sound_sink: Sink,
}

pub struct SoundPort2 {
    last_value: u8,
    device: Device,
    instant_sound_4: String,
    instant_sound_5: String,
    instant_sound_6: String,
    instant_sound_7: String,
    instant_sound_8: String,
    sound_sink: Sink,
}

fn create_sound(path: &str) -> std::io::Result<File> {
    File::open(path)
}

impl SoundPort1 {
    pub fn new(folder: &str) -> Result<SoundPort1, Error> {
        let device = rodio::default_output_device().unwrap();
        Ok(SoundPort1 {
            last_value: 0,
            background: {
                let sink = Sink::new(&device);
                let sound = create_sound(&format!("{}/0.wav", folder))?;
                let sound = Decoder::new(BufReader::new(sound))
                    .map_err(|e| Error::from(ConsoleError::CantCreateSound { msg: e.to_string() }))?;
                sink.append(sound.repeat_infinite());
                sink.stop();
                sink
            },
            instant_sound_1: format!("{}/1.wav", folder),
            instant_sound_2: format!("{}/2.wav", folder),
            instant_sound_3: format!("{}/3.wav", folder),
            sound_sink: Sink::new(&device),
            device,
        })
    }
}

impl SoundPort2 {
    pub fn new(folder: &str) -> Result<SoundPort2, Error> {
        let device = rodio::default_output_device().unwrap();
        Ok(SoundPort2 {
            last_value: 0,
            instant_sound_4: format!("{}/4.wav", folder),
            instant_sound_5: format!("{}/5.wav", folder),
            instant_sound_6: format!("{}/6.wav", folder),
            instant_sound_7: format!("{}/7.wav", folder),
            instant_sound_8: format!("{}/8.wav", folder),
            sound_sink: Sink::new(&device),
            device,
        })
    }
}

macro_rules! maybe_play_instant_sound {
    ($position:expr, $byte:ident, $this:ident, $sound:ident) => {
        if ($byte & $position) ^ ($byte & $this.last_value) > 0 && $this.sound_sink.empty() {
            let file = create_sound(&$this.$sound).unwrap();
            let sound = Decoder::new(BufReader::new(file))
                .map_err(|e| Error::from(ConsoleError::CantCreateSound { msg: e.to_string() }))
                .unwrap();
            $this.sound_sink.append(sound);
            $this.sound_sink.play();
        }
    };
}
impl OutputDevice for SoundPort1 {
    fn write(&mut self, byte: u8) {
        if (byte & 0x01) ^ (byte & self.last_value) > 0 {
            if !self.background.empty() {
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
