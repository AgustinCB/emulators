use mos6502cpu::{AddressingMode, Cpu, Mos6502Cpu, Mos6502Instruction, Mos6502InstructionCode};
use ppu::Ppu;
use ram::{Ram, ROM_SIZE};
use std::cell::RefCell;
use std::rc::Rc;
use super::failure::Error;

pub(crate) trait InputOutputDevice {
    fn read(&self) -> u8;
    fn write(&mut self, value: u8) -> u8;
}

pub struct Nes {
    cpu: Mos6502Cpu,
    pub ram: Rc<RefCell<Ram>>,
    ppu: Ppu,
}

impl Nes {
    pub fn new(rom: [u8; ROM_SIZE]) -> Nes {
        let ram = Rc::new(RefCell::new(Ram::new(rom)));
        let cpu = Mos6502Cpu::without_decimal(Box::new(ram.clone()));
        let ppu = Ppu::new(ram.clone());
        Nes {
            cpu,
            ppu,
            ram,
        }
    }

    pub fn power_up(&mut self) -> Result<(), Error> {
        self.cpu.execute_instruction(&Mos6502Instruction::new(
            Mos6502InstructionCode::Rst,
            AddressingMode::Implicit,
        ))
    }
}