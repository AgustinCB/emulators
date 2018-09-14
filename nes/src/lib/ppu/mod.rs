mod ppu;
mod register_2000;
mod register_2001;
mod register_2002;
mod register_4014;

pub(crate) enum SpriteMode {
    EightEight,
    EightSixteen,
}

pub(crate) enum PpuMode {
    Master,
    Slave,
}

pub use self::ppu::Ppu;