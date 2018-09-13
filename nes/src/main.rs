extern crate failure;
extern crate mos6502cpu;
extern crate nes;

use failure::Error;
use mos6502cpu::*;
use nes::{Nes, ROM_SIZE};
use std::env::args;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: nes [game file]";

fn read_file(file_name: &str) -> std::io::Result<[u8; ROM_SIZE]> {
    let mut f = File::open(file_name)?;
    let mut memory = [0; ROM_SIZE];
    f.read(&mut memory)?;
    Ok(memory)
}

fn start_game(game: &str) -> Result<(), Error> {
    let rom = read_file(game)?;
    let nes = Nes::new(rom);
    let ram = &mut *(nes.ram.borrow_mut());
    let _cpu = Mos6502Cpu::new(ram);
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        panic!(USAGE);
    }
    start_game(&args[1]).unwrap();
}
