extern crate intel8080cpu;
extern crate emulator_space_invaders;
extern crate failure;

use intel8080cpu::*;
use emulator_space_invaders::console::{ConsoleOptions, Console};
use failure::Error;
use std::env::args;
use std::fs::File;
use std::io::Read;

const USAGE: &'static str = "Usage: disassembler-8080 [game|test] [file] [--no-audio]

If running either test, [file] should be a hex file with Intel 8080 instructions.

When selecting the mode game, [file] should be a folder that contains the following content:

./rom # The rom of the game
./0.wav ... 9.wav # The audio files of the game";

struct PrintScreen;

impl Printer for PrintScreen {
    fn print(&mut self, bytes: &[u8]) {
        println!("{}", String::from_utf8_lossy(bytes));
    }
}

fn read_file(file_name: &str) -> std::io::Result<[u8; ROM_MEMORY_LIMIT]> {
    let mut f = File::open(file_name)?;
    // this may blow up memory if the file is big enough
    // TODO: streams???
    let mut memory = [0; ROM_MEMORY_LIMIT];
    f.read(&mut memory)?;
    Ok(memory)
}

fn start_game(folder: &str, has_audio: bool) -> Result<(), Error> {
    let rom_location = format!("{}/rom", folder);
    let memory = read_file(&rom_location)?;
    let options = ConsoleOptions::new(memory, folder).with_audio(has_audio);
    let mut console = Console::new(options)?;
    console.start().map_err(|e| Error::from(e))
}

fn test(memory: [u8; ROM_MEMORY_LIMIT]) -> Result<(), Error> {
    let screen = &mut (PrintScreen {});
    let mut cpu = Intel8080Cpu::new_cp_m_compatible(memory, screen);

    while !cpu.is_done() {
        cpu.execute()?;
    }
    Ok(())
}

fn handle_result(r: Result<(), Error>) {
    match r {
        Ok(()) => {},
        Err(err) => panic!(err),
    };
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 && args.len() != 4 {
        panic!(USAGE);
    }

    if args[1] == "game" {
        let has_audio = if args.len() == 4 {
            args[3] != String::from("--no-audio")
        } else {
            true
        };
        handle_result(start_game(&args[2], has_audio));
    } else if args[1] == "test" {
        let memory = read_file(&args[2]).unwrap();
        handle_result(test(memory));
    } else {
        panic!(USAGE);
    }
}
