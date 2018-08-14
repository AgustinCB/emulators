mod buttons;
mod external_shift;
mod sounds;
use super::cpu::{InputDevice, OutputDevice};

pub struct DummyOutputDevice {}

impl OutputDevice for DummyOutputDevice {
    fn write(&mut self, _: u8) {}
}

pub struct DummyInputDevice {
    pub value: u8,
}

impl InputDevice for DummyInputDevice {
    fn read(&mut self) -> u8 {
        self.value
    }
}

pub use self::external_shift::*;
pub use self::buttons::*;
pub use self::sounds::*;