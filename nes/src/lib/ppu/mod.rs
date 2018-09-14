mod ppu;
mod register_2000;
mod register_2001;
mod register_2002;
mod address_register;
mod register_4014;

pub(crate) enum SpriteMode {
    EightEight,
    EightSixteen,
}

pub(crate) enum PpuMode {
    Master,
    Slave,
}

pub(crate) enum ColorMode {
    Color,
    Monochrome,
}

pub use self::ppu::Ppu;