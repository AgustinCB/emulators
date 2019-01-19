mod address_register;
mod ppu;
mod register_2000;
mod register_2001;
mod register_2002;
mod register_2004;
mod register_2007;
mod register_4014;
mod video_ram;

pub(crate) type SpriteMemory = [u8; 256];

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
