extern crate failure;

use failure::{Error, Fail};

pub trait MemoryAddressWidth {}

impl MemoryAddressWidth for u8 {}
impl MemoryAddressWidth for u16 {}
impl MemoryAddressWidth for u32 {}
impl MemoryAddressWidth for u64 {}

pub enum Cycles {
    Single(u8),
    Conditional { not_met: u8, met: u8 },
}

pub trait InputDevice {
    fn read(&mut self) -> u8;
}

pub trait OutputDevice {
    fn write(&mut self, byte: u8);
}

pub trait Instruction<W: MemoryAddressWidth, E: Fail> {
    fn size(&self) -> Result<u8, E>;
    fn get_cycles(&self) -> Result<Cycles, E>;
}

pub trait Cpu<W, I, E, F>
    where W: MemoryAddressWidth + Clone,
          I: Instruction<W, F> + ToString + From<Vec<W>>,
          F: Fail,
          E: Fail {
    fn execute(&mut self) -> Result<u8, Error> {
        let instruction = I::from(self.get_next_instruction_bytes().to_vec());
        if !self.can_run(&instruction) {
            return Ok(0);
        }
        println!("{}", instruction.to_string());
        self.increase_pc(instruction.size()?);
        let cycles = self.get_cycles_for_instruction(&instruction)?;
        self.execute_instruction(instruction)?;
        Ok(cycles)
    }

    fn get_cycles_for_instruction(&self, instruction: &I) -> Result<u8, F> {
        let cycles = instruction.get_cycles()?;
        Ok(match cycles {
            Cycles::Single(cycles) => cycles,
            Cycles::Conditional { not_met, met } =>
                self.get_cycles_from_condition(instruction, not_met, met),
        })
    }

    fn execute_instruction(&mut self, instruction: I) -> Result<(), E>;
    fn get_next_instruction_bytes(&self) -> &[W];
    fn can_run(&self, instruction: &I) -> bool;
    fn is_done(&self) -> bool;
    fn add_input_device(&mut self, id: u8, device: Box<InputDevice>);
    fn add_output_device(&mut self, id: u8, device: Box<OutputDevice>);
    fn increase_pc(&mut self, steps: u8);
    fn get_cycles_from_condition(&self, instruction: &I, not_met: u8, met: u8) -> u8;
}