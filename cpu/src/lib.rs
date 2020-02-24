#![macro_use]
#![no_std]

extern crate alloc;
extern crate failure;

use alloc::boxed::Box;
use alloc::vec::Vec;
use failure::{Error, Fail};

#[macro_export]
macro_rules! single {
    ($num:expr) => {
        Cycles::Single($num)
    };
}

#[macro_export]
macro_rules! conditional {
    ($not_met:expr, $met:expr) => {
        Cycles::OneCondition {
            not_met: $not_met,
            met: $met,
        }
    };
}

#[macro_export]
macro_rules! bi_conditional {
    ($not_met:expr, $first_met:expr, $second_met:expr) => {
        Cycles::TwoConditions {
            not_met: $not_met,
            first_met: $first_met,
            second_met: $second_met,
        }
    };
}

pub enum Cycles {
    Single(u8),
    OneCondition {
        not_met: u8,
        met: u8,
    },
    TwoConditions {
        not_met: u8,
        first_met: u8,
        second_met: u8,
    },
}

pub trait InputDevice {
    fn read(&mut self) -> u8;
}

pub trait OutputDevice {
    fn write(&mut self, byte: u8);
}

pub trait Instruction {
    fn size(&self) -> Result<u8, Error>;
    fn get_cycles(&self) -> Result<Cycles, Error>;
}

pub trait Cpu<I, F>
where
    I: Instruction + From<Vec<u8>>,
    F: Fail,
{
    fn execute(&mut self) -> Result<u8, Error> {
        let instruction = I::from(self.get_next_instruction_bytes());
        if !self.can_run(&instruction) {
            return Ok(0);
        }
        self.increase_pc(instruction.size()?);
        self.execute_instruction(&instruction)?;
        let cycles = self.get_cycles_for_instruction(&instruction)?;
        Ok(cycles)
    }

    fn get_cycles_for_instruction(&mut self, instruction: &I) -> Result<u8, Error> {
        let cycles = instruction.get_cycles()?;
        match cycles {
            Cycles::Single(cycles) => Ok(cycles),
            Cycles::OneCondition { not_met, met } => {
                self.get_cycles_from_one_condition(instruction, not_met, met)
            }
            Cycles::TwoConditions {
                not_met,
                first_met,
                second_met,
            } => self.get_cycles_from_two_conditions(instruction, not_met, first_met, second_met),
        }
    }

    fn execute_instruction(&mut self, instruction: &I) -> Result<(), Error>;
    fn get_pc(&self) -> u16;
    fn get_next_instruction_bytes(&self) -> Vec<u8>;
    fn can_run(&self, instruction: &I) -> bool;
    fn is_done(&self) -> bool;
    fn increase_pc(&mut self, steps: u8);
    fn get_cycles_from_one_condition(
        &self,
        instruction: &I,
        not_met: u8,
        met: u8,
    ) -> Result<u8, Error>;
    fn get_cycles_from_two_conditions(
        &self,
        instruction: &I,
        not_met: u8,
        first_met: u8,
        second_met: u8,
    ) -> Result<u8, Error>;
}

pub trait WithPorts {
    fn add_input_device(&mut self, id: u8, device: Box<dyn InputDevice>);
    fn add_output_device(&mut self, id: u8, device: Box<dyn OutputDevice>);
}
