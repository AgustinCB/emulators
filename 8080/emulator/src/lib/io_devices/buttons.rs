extern crate piston;

use self::piston::input::Key;
use super::super::cpu::InputDevice;
use std::cell::RefCell;
use std::rc::Rc;

enum GameButton {
    Coin,
    Left,
    Right,
    Fire,
    Start,
}

pub struct KeypadController {
    buttons_pressed: Rc<RefCell<u8>>,
}

impl KeypadController {
    pub fn new() -> KeypadController {
        KeypadController {
            buttons_pressed: Rc::new(RefCell::new(0x08)),
        }
    }

    pub fn buttons_pressed(&self) -> Rc<RefCell<u8>> {
        self.buttons_pressed.clone()
    }

    pub fn key_pressed(&mut self, key: Key) {
        let button = self.game_button_from_key(key);
        let mut result = *self.buttons_pressed.borrow();
        match button {
            Some(GameButton::Coin) => result |= 0x01,
            Some(GameButton::Start) => result |= 0x04,
            Some(GameButton::Fire) => result |= 0x10,
            Some(GameButton::Left) => result |= 0x20,
            Some(GameButton::Right) => result |= 0x40,
            _ => {},
        };
        *(self.buttons_pressed.borrow_mut()) = result;
    }

    pub fn key_released(&mut self, key: Key) {
        let button = self.game_button_from_key(key);
        let mut result = *(self.buttons_pressed.borrow());
        match button {
            Some(GameButton::Coin) => result &= !0x01,
            Some(GameButton::Start) => result &= !0x04,
            Some(GameButton::Fire) => result &= !0x10,
            Some(GameButton::Left) => result &= !0x20,
            Some(GameButton::Right) => result &= !0x40,
            _ => {},
        };
        *(self.buttons_pressed.borrow_mut()) = result;
    }

    #[inline]
    fn game_button_from_key(&self, key: Key) -> Option<GameButton> {
        match key {
            Key::A => Some(GameButton::Left),
            Key::S => Some(GameButton::Right),
            Key::Space => Some(GameButton::Start),
            Key::C => Some(GameButton::Coin),
            Key::F => Some(GameButton::Fire),
            _ => None,
        }
    }
}

pub struct KeypadInput {
    buttons_pressed: Rc<RefCell<u8>>,
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
        *(self.buttons_pressed).borrow()
    }
}