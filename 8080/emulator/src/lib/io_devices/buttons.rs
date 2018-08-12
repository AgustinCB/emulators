extern crate piston;

use self::piston::input::Key;
use super::super::cpu::InputDevice;
use std::cell::Cell;

enum GameButton {
    Coin,
    Left,
    Right,
    Fire,
    Start,
}

pub struct KeypadController {
    buttons_pressed: Cell<u8>,
}

impl KeypadController {
    pub fn new() -> KeypadController {
        KeypadController {
            buttons_pressed: Cell::new(0x08),
        }
    }

    pub fn buttons_pressed(&self) -> Cell<u8> {
        Cell::clone(&self.buttons_pressed)
    }

    pub fn key_pressed(&mut self, key: Key) {
        let button = self.game_button_from_key(key);
        let mut result = self.buttons_pressed.get();
        match button {
            Some(GameButton::Coin) => result |= 0x01,
            Some(GameButton::Start) => result |= 0x04,
            Some(GameButton::Fire) => result |= 0x10,
            Some(GameButton::Left) => result |= 0x20,
            Some(GameButton::Right) => result |= 0x40,
            _ => {},
        };
        self.buttons_pressed.set(result);
    }

    pub fn key_released(&mut self, key: Key) {
        let button = self.game_button_from_key(key);
        let mut result = self.buttons_pressed.get();
        match button {
            Some(GameButton::Coin) => result &= !0x01,
            Some(GameButton::Start) => result &= !0x04,
            Some(GameButton::Fire) => result &= !0x10,
            Some(GameButton::Left) => result &= !0x20,
            Some(GameButton::Right) => result &= !0x40,
            _ => {},
        };
        self.buttons_pressed.set(result);
    }

    #[inline]
    fn game_button_from_key(&self, key: Key) -> Option<GameButton> {
        match key {
            Key::A => Some(GameButton::Left),
            Key::S => Some(GameButton::Right),
            Key::Insert => Some(GameButton::Start),
            Key::C => Some(GameButton::Coin),
            Key::Space => Some(GameButton::Fire),
            _ => None,
        }
    }
}

pub struct KeypadInput {
    buttons_pressed: Cell<u8>,
}

impl KeypadInput {
    pub fn new(controller: &KeypadController) -> KeypadInput {
        KeypadInput {
            buttons_pressed: controller.buttons_pressed(),
        }
    }
}

impl InputDevice for KeypadInput {
    fn read(&mut self) -> u8 {
        self.buttons_pressed.get()
    }
}