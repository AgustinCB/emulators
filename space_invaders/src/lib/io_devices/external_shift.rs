use super::intel8080cpu::{InputDevice, OutputDevice};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ExternalShiftOffsetWriter {
    shift_offset: Rc<RefCell<u8>>,
}

impl ExternalShiftOffsetWriter {
    pub fn new() -> ExternalShiftOffsetWriter {
        ExternalShiftOffsetWriter {
            shift_offset: Rc::new(RefCell::new(0)),
        }
    }

    pub fn get_shift_offset(&self) -> Rc<RefCell<u8>> {
        self.shift_offset.clone()
    }
}

impl OutputDevice for ExternalShiftOffsetWriter {
    fn write(&mut self, value: u8) {
        *(self.shift_offset.borrow_mut()) = value & 0x07;
    }
}

pub struct ExternalShiftWriter {
    shift0: Rc<RefCell<u8>>,
    shift1: Rc<RefCell<u8>>,
}

impl OutputDevice for ExternalShiftWriter {
    fn write(&mut self, value: u8) {
        *(self.shift1.borrow_mut()) = *(self.shift0.borrow());
        *(self.shift0.borrow_mut()) = value;
    }
}

impl ExternalShiftWriter {
    pub fn new() -> ExternalShiftWriter {
        ExternalShiftWriter {
            shift0: Rc::new(RefCell::new(0)),
            shift1: Rc::new(RefCell::new(0)),
        }
    }

    pub fn get_shift0(&self) -> Rc<RefCell<u8>> {
        self.shift0.clone()
    }

    pub fn get_shift1(&self) -> Rc<RefCell<u8>> {
        self.shift1.clone()
    }
}

pub struct ExternalShiftReader {
    shift_offset: Rc<RefCell<u8>>,
    shift0: Rc<RefCell<u8>>,
    shift1: Rc<RefCell<u8>>,
}

impl InputDevice for ExternalShiftReader {
    fn read(&mut self) -> u8 {
        let v = (u16::from(*self.shift0.borrow()) << 8) | u16::from(*self.shift1.borrow());
        (v >> (8 - *self.shift_offset.borrow())) as u8
    }
}

impl ExternalShiftReader {
    pub fn new(
        shift_writer: &ExternalShiftWriter,
        offset_writer: &ExternalShiftOffsetWriter,
    ) -> ExternalShiftReader {
        ExternalShiftReader {
            shift_offset: offset_writer.get_shift_offset(),
            shift0: shift_writer.get_shift0(),
            shift1: shift_writer.get_shift1(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_perform_shift() {
        let mut shift_writer = ExternalShiftWriter::new();
        let mut offset_writer = ExternalShiftOffsetWriter::new();
        let mut shift_reader = ExternalShiftReader::new(&shift_writer, &offset_writer);

        shift_writer.write(0);
        shift_writer.write(1);
        offset_writer.write(6);

        assert_eq!(shift_reader.read(), 64);
    }
}
