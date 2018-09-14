mod ppu;
mod register_2000;
mod register_2001;

pub(crate) enum SpriteMode {
    EightEight,
    EightSixteen,
}

pub use self::ppu::Ppu;