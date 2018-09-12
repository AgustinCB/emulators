extern crate mos6502cpu;
extern crate failure;

use mos6502cpu::*;
use failure::Error;
use std::env::args;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: nes [game file]";

fn read_file(file_name: &str) -> std::io::Result<[u8; AVAILABLE_MEMORY]> {
    let mut f = File::open(file_name)?;
    let mut memory = [0; AVAILABLE_MEMORY];
    f.read(&mut memory)?;
    Ok(memory)
}

fn start_game(game: &str) -> Result<(), Error> {
    let memory = read_file(game)?;
    let _cpu = Mos6502Cpu::new(memory);
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!(USAGE);
    }
    start_game(&args[1]).unwrap();
}
