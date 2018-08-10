extern crate termion;

use self::termion::event::{Event, Key};
use self::termion::input::TermRead;
use super::cpu::{InputDevice, OutputDevice};
use std::borrow::BorrowMut;
use std::cell::Cell;
use std::collections::HashSet;
use std::io::{Error, stdin};

const NUM_BUTTONS: usize = 5;
#[derive(Hash, PartialEq, Eq)]
enum GameButton {
    Coin,
    Left,
    Right,
    Fire,
    Start,
}

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

pub(crate) struct ExternalShiftOffsetWriter {
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
        self.shift_offset.borrow_mut().set(value & 0x7);
    }
}

pub(crate) struct ExternalShiftWriter {
    shift0: Cell<u8>,
    shift1: Cell<u8>,
}

impl OutputDevice for ExternalShiftWriter {
    fn write(&mut self, value: u8) {
        self.shift0.borrow_mut().set(self.shift1.get());
        self.shift1.borrow_mut().set(value);
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

pub(crate) struct ExternalShiftReader {
    shift_offset: Cell<u8>,
    shift0: Cell<u8>,
    shift1: Cell<u8>,
}

impl InputDevice for ExternalShiftReader {
    fn read(&mut self) -> u8 {
        let v = ((self.shift1.get() as u16) << 8) as u8 | self.shift0.get();
        ((v >> (8-self.shift_offset.get())) & 0xff)
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

pub struct KeypadInput {}

impl KeypadInput {
    fn get_keys() -> Result<HashSet<GameButton>, Error> {
        let mut result = HashSet::with_capacity(NUM_BUTTONS);
        for c in stdin().events() {
            let evt = c?;
            match evt {
                Event::Key(Key::Char('a')) => result.insert(GameButton::Left),
                Event::Key(Key::Char('s')) => result.insert(GameButton::Right),
                Event::Key(Key::Char('c')) => result.insert(GameButton::Coin),
                Event::Key(Key::Char(' ')) => result.insert(GameButton::Start),
                Event::Key(Key::Char('f')) => result.insert(GameButton::Fire),
                _ => true
            };
        }
        Ok(result)
    }
}

impl InputDevice for KeypadInput {
    fn read(&mut self) -> u8 {
        let mut result = 0x08;
        match KeypadInput::get_keys() {
            Ok(buttons_pressed) => {
                if buttons_pressed.contains(&GameButton::Coin) {
                    result |= 0x01;
                }
                if buttons_pressed.contains(&GameButton::Start) {
                    result |= 0x04;
                }
                if buttons_pressed.contains(&GameButton::Fire) {
                    result |= 0x10;
                }
                if buttons_pressed.contains(&GameButton::Left) {
                    result |= 0x20;
                }
                if buttons_pressed.contains(&GameButton::Right) {
                    result |= 0x40;
                }
            },
            Err(e) => panic!(e),
        }
        result
    }
}