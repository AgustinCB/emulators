extern crate piston;

use self::piston::input::Key;
use super::super::cpu::InputDevice;
use std::collections::HashSet;

const NUM_BUTTONS: usize = 5;
#[derive(Hash, PartialEq, Eq)]
enum GameButton {
    Coin,
    Left,
    Right,
    Fire,
    Start,
}

pub struct KeypadInput {
    buttons_pressed: HashSet<GameButton>,
}

impl KeypadInput {
    pub fn new() -> KeypadInput {
        KeypadInput {
            buttons_pressed: HashSet::with_capacity(NUM_BUTTONS),
        }
    }

    pub fn key_pressed(&mut self, key: Key) {
        match key {
            Key::A => self.buttons_pressed.insert(GameButton::Left),
            Key::S => self.buttons_pressed.insert(GameButton::Right),
            Key::Insert => self.buttons_pressed.insert(GameButton::Start),
            Key::C => self.buttons_pressed.insert(GameButton::Coin),
            Key::Space => self.buttons_pressed.insert(GameButton::Fire),
            _ => false,
        };
    }

    pub fn key_released(&mut self, key: Key) {
        match key {
            Key::A => self.buttons_pressed.remove(&GameButton::Left),
            Key::S => self.buttons_pressed.remove(&GameButton::Right),
            Key::Insert => self.buttons_pressed.remove(&GameButton::Start),
            Key::C => self.buttons_pressed.remove(&GameButton::Coin),
            Key::Space => self.buttons_pressed.remove(&GameButton::Fire),
            _ => false,
        };
    }
}

impl InputDevice for KeypadInput {
    fn read(&mut self) -> u8 {
        let mut result = 0x08;
        if self.buttons_pressed.contains(&GameButton::Coin) {
            result |= 0x01;
        }
        if self.buttons_pressed.contains(&GameButton::Start) {
            result |= 0x04;
        }
        if self.buttons_pressed.contains(&GameButton::Fire) {
            result |= 0x10;
        }
        if self.buttons_pressed.contains(&GameButton::Left) {
            result |= 0x20;
        }
        if self.buttons_pressed.contains(&GameButton::Right) {
            result |= 0x40;
        }
        result
    }
}