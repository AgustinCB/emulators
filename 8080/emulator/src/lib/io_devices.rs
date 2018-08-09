use super::cpu::{InputDevice, OutputDevice};

pub struct DummyOutputDevice {}

impl OutputDevice for DummyOutputDevice {
    fn write(&mut self, _: u8, _: u8) {}
}

pub struct DummyInputDevice {
    pub value: u8,
}

impl InputDevice for DummyInputDevice {
    fn read(&mut self, _: u8) -> u8 {
        self.value
    }
}

pub struct ExternalShift {
    shift_offset: u8,
    shift0: u8,
    shift1: u8,
}

impl ExternalShift {
    pub fn new() -> ExternalShift {
        ExternalShift {
            shift0: 0,
            shift1: 0,
            shift_offset: 0,
        }
    }
}

impl InputDevice for ExternalShift {
    fn read(&mut self, _: u8) -> u8 {
        let v = ((self.shift1 as u16) << 8) as u8 | self.shift0;
        ((v >> (8-self.shift_offset)) & 0xff)
    }
}

impl OutputDevice for ExternalShift {
    fn write(&mut self, port: u8, value: u8) {
        if port == 2 {
            self.shift_offset = value & 0x7;
        } else if port == 4 {
            self.shift0 = self.shift1;
            self.shift1 = value;
        }
    }
}