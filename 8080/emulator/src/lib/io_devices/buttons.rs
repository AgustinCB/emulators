extern crate termion;

use self::termion::event::{Event, Key};
use self::termion::input::TermRead;
use super::super::cpu::InputDevice;
use std::collections::HashSet;
use std::io::{Error, Read, stdin};

const NUM_BUTTONS: usize = 5;
#[derive(Hash, PartialEq, Eq)]
enum GameButton {
    Coin,
    Left,
    Right,
    Fire,
    Start,
}

pub struct KeypadInput {}

impl KeypadInput {
    fn get_keys() -> Result<HashSet<GameButton>, Error> {
        let mut result = HashSet::with_capacity(NUM_BUTTONS);
        let mut buff = [0; 5];
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