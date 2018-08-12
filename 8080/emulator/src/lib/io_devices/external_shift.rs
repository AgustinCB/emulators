use super::super::cpu::{InputDevice, OutputDevice};
use std::cell::Cell;


pub struct ExternalShiftOffsetWriter {
    shift_offset: Cell<u8>,
}

impl ExternalShiftOffsetWriter {
    pub fn new() -> ExternalShiftOffsetWriter {
        ExternalShiftOffsetWriter {
            shift_offset: Cell::new(0),
        }
    }

    pub fn get_shift_offset(&self) -> Cell<u8> {
        self.shift_offset.clone()
    }
}

impl OutputDevice for ExternalShiftOffsetWriter {
    fn write(&mut self, value: u8) {
        self.shift_offset.set(value & 0x7);
    }
}

pub struct ExternalShiftWriter {
    shift0: Cell<u8>,
    shift1: Cell<u8>,
}

impl OutputDevice for ExternalShiftWriter {
    fn write(&mut self, value: u8) {
        self.shift0.set(self.shift1.get());
        self.shift1.set(value);
    }
}

impl ExternalShiftWriter {
    pub fn new() -> ExternalShiftWriter {
        ExternalShiftWriter {
            shift0: Cell::new(0),
            shift1: Cell::new(0),
        }
    }

    pub fn get_shift0(&self) -> Cell<u8> {
        self.shift0.clone()
    }

    pub fn get_shift1(&self) -> Cell<u8> {
        self.shift1.clone()
    }
}

pub struct ExternalShiftReader {
    shift_offset: Cell<u8>,
    shift0: Cell<u8>,
    shift1: Cell<u8>,
}

impl InputDevice for ExternalShiftReader {
    fn read(&mut self) -> u8 {
        let v = ((self.shift1.get() as u16) << 8) as u8 | self.shift0.get();
        (((v as u16) >> (8-self.shift_offset.get())) & 0xff) as u8
    }
}

impl ExternalShiftReader {
    pub fn new(shift_writer: &ExternalShiftWriter, offset_writer: &ExternalShiftOffsetWriter)
               -> ExternalShiftReader {
        ExternalShiftReader {
            shift_offset: offset_writer.get_shift_offset(),
            shift0: shift_writer.get_shift0(),
            shift1: shift_writer.get_shift1(),
        }
    }
}