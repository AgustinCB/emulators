extern crate rodio;

use std::fs::File;
use std::io::{BufReader, Error};
use self::rodio::{default_output_device, Decoder, Device, Sink, Source};

struct Sounds {
    instant_sound_1: File,
    instant_sound_2: File,
    instant_sound_3: File,
    instant_sound_4: File,
    instant_sound_5: File,
    instant_sound_6: File,
    instant_sound_7: File,
    instant_sound_8: File,
    background: File,
}

impl Sounds {
    fn new(folder: &str) -> Result<Sounds, String> {
        Ok(Sounds {
            background: Sounds::create_source(&format!("{}/0.wav", folder))?,
            instant_sound_1: Sounds::create_source(&format!("{}/1.wav", folder))?,
            instant_sound_2: Sounds::create_source(&format!("{}/2.wav", folder))?,
            instant_sound_3: Sounds::create_source(&format!("{}/3.wav", folder))?,
            instant_sound_4: Sounds::create_source(&format!("{}/4.wav", folder))?,
            instant_sound_5: Sounds::create_source(&format!("{}/5.wav", folder))?,
            instant_sound_6: Sounds::create_source(&format!("{}/6.wav", folder))?,
            instant_sound_7: Sounds::create_source(&format!("{}/7.wav", folder))?,
            instant_sound_8: Sounds::create_source(&format!("{}/8.wav", folder))?,
        })
    }

    fn create_source(file_location: &str) -> Result<File, String> {
        File::open(file_location)
            .map_err(|e| e.to_string())
    }
}

pub struct SoundOutput {
    device: Device,
    last_value: u8,
    channel1: Sink,
    channel2: Sink,
    sounds: Sounds,
}

impl SoundOutput {
    pub fn new(folder: &str) -> Result<SoundOutput, String> {
        let device = default_output_device()
            .ok_or("No sound")?;
        let sink1 = Sink::new(&device);
        let sink2 = Sink::new(&device);
        let sounds = Sounds::new(folder)?;
        Ok(SoundOutput {
            device,
            last_value: 0,
            channel1: sink1,
            channel2: sink2,
            sounds,
        })
    }

    fn play(&self, file: &'static File) -> Result<(), String> {
        let source = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
        self.channel1.append(source);
        if self.channel1.empty() {
            self.channel1.play();
        }
        Ok(())
    }

    fn repeat(&self, file: &'static File) -> Result<(), String> {
        let source = Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
        self.channel2.stop();
        self.channel2.append(source.repeat_infinite());
        self.channel2.play();
        Ok(())
    }

    fn stop_repeat(&self) {
        self.channel2.stop();
    }
}