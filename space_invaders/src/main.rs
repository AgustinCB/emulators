extern crate emulator_space_invaders;
extern crate failure;
extern crate find_folder;
extern crate intel8080cpu;

use emulator_space_invaders::console::{Console, ConsoleOptions};
use emulator_space_invaders::view::View;
use failure::Error;
use intel8080cpu::*;
use std::env::args;
use std::fs::File;
use std::io::Read;

const USAGE: &str = "Usage: space-invaders [game|test] [file] [--no-audio]

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
    f.read_exact(&mut memory)?;
    Ok(memory)
}

fn start_game(folder: &str, has_audio: bool, debug: bool) -> Result<(), Error> {
    let rom_location = format!("{}/rom", folder);
    let memory = read_file(&rom_location)?;
    let options = ConsoleOptions::new(memory, folder).with_audio(has_audio);
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets")
        .unwrap();
    let mut window = Console::create_window(debug)?;
    let glyphs = window.load_font(assets.join("FiraSans-Regular.ttf"))?;
    let view = View::new(debug, glyphs);
    let mut console = Console::new(options, view, window)?;
    console.start().map_err(Error::from)
}

fn test(memory: [u8; ROM_MEMORY_LIMIT]) -> Result<(), Error> {
    let screen = &mut (PrintScreen {});
    let mut cpu = Intel8080Cpu::new_cp_m_compatible(memory, screen);

    while !cpu.is_done() {
        cpu.execute()?;
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 3 || args.len() > 5 {
        panic!(USAGE);
    }

    if args[1] == "game" {
        let has_audio = !args.iter().find(|a| a.as_str() == "--no-audio").is_some();
        let debug = args.iter().find(|a| a.as_str() == "--debug").is_some();
        start_game(&args[2], has_audio, debug).unwrap();
    } else if args[1] == "test" {
        let memory = read_file(&args[2]).unwrap();
        test(memory).unwrap();
    } else {
        panic!(USAGE);
    }
}
