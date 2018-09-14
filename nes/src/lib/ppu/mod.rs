mod ppu;
mod register_2000;

pub(crate) enum SpriteMode {
    EightEight,
    EightSixteen,
}

pub use self::ppu::Ppu;