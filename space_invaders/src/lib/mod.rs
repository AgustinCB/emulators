#[macro_use] extern crate failure;

#[derive(Debug, Fail)]
pub enum ConsoleError {
    #[fail(display = "couldn't create cpu: {}", msg)]
    CantCreateCpu {
        msg: String,
    },
    #[fail(display = "couldn't create window: {}", msg)]
    CantCreateWindow {
        msg: String,
    },
    #[fail(display = "couldn't create sound: {}", msg)]
    CantCreateSound {
        msg: String,
    },
}

pub mod console;
mod io_devices;
mod screen;
mod timer;
mod view;