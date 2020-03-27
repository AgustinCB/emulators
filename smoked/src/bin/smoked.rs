use smoked::cpu::VM;
use std::env::args;
use std::fs::File;
use std::io::prelude::*;
use cpu::Cpu;

const USAGE: &str = "Usage: smoked [-s] [input file]";

struct Config {
    input_file: Option<String>,
    show_stack: bool,
}

fn parse_config<I: Iterator<Item=String>>(mut strings: I) -> Config {
    let mut configuration = Config {
        input_file: None,
        show_stack: false,
    };
    while let Some(next) = strings.next() {
        match next.as_str() {
            "-s" | "--show-stack" => {
                configuration.show_stack = true;
            }
            s if configuration.input_file.is_none() => {
                configuration.input_file = Some(s.to_owned());
            }
            _ => panic!(USAGE),
        }
    }
    configuration
}

fn main () {
    let conf = parse_config(args());
    let mut input_file = File::create(
        conf.input_file.unwrap_or("/dev/stdout".to_owned())
    ).unwrap();
    let mut bytes = vec![];
    input_file.read_to_end(&mut bytes).unwrap();
    let mut vm = VM::from(bytes.as_ref());
    while !vm.is_done() {
        vm.execute().unwrap();
    }
    if conf.show_stack {
        for (index, value) in vm.stack().iter().rev().enumerate() {
            println!("{} - {:?}", index, value);
        }
    }
}