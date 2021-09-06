use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use smoked::serde::from_bytes;
use std::str::FromStr;

const USAGE: &str = "Usage: smoked [-s] [-d] [input file]";

#[derive(Debug)]
struct Config {
    debug: bool,
    input_file: Option<String>,
    show_instructions: bool,
    show_stack: bool,
    stack_size: Option<usize>,
}

fn parse_config<I: Iterator<Item = String>>(mut strings: I) -> Config {
    let mut configuration = Config {
        debug: false,
        input_file: None,
        show_instructions: false,
        show_stack: false,
        stack_size: None,
    };
    strings.next();
    while let Some(next) = strings.next() {
        match next.as_str() {
            "-d" | "--debug" => {
                configuration.debug = true;
            }
            "-i" | "--show-instructions" => {
                configuration.show_instructions = true;
            }
            "-s" | "--show-stack" => {
                configuration.show_stack = true;
            }
            "-S" | "--stack-size" => {
                let string_number = strings.next().unwrap();
                let number = usize::from_str(&string_number).unwrap();
                configuration.stack_size = Some(number);
            }
            s if configuration.input_file.is_none() => {
                configuration.input_file = Some(s.to_owned());
            }
            _ => panic!("{}", USAGE),
        }
    }
    configuration
}

fn main() {
    let conf = parse_config(args());
    let mut input_file: Box<dyn Read> = conf
        .input_file.clone()
        .map::<Box<dyn Read>, _>(|f| Box::new(File::create(f).unwrap()))
        .unwrap_or_else(|| Box::new(std::io::stdin()));
    let mut bytes = vec![];
    input_file.read_to_end(&mut bytes).unwrap();
    let mut vm = from_bytes(bytes.as_ref(), conf.stack_size.clone());
    vm.debug = conf.show_instructions;
    if conf.debug {
        eprintln!("Constants: {:?}", vm.constants);
        eprintln!("Instructions: {:?}", vm.rom);
        eprintln!("Locations: {:?}", vm.locations);
    }
    while !vm.is_done() {
        if let Err(e) = vm.execute() {
            eprintln!("{}", e);
            break;
        }
    }
    if conf.show_stack {
        for (index, value) in vm.stack().iter().rev().enumerate() {
            println!("{} - {:?}", index, value);
        }
    }
}
