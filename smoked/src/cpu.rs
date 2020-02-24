use crate::allocator::Allocator;
use crate::memory::Memory;
use cpu::Cpu;
use crate::instruction::Instruction;
use failure::Error;
use log::debug;
use std::cmp::min;

const STACK_MAX: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(usize),
    Nil,
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
            Value::String(_) => true,
            Value::Nil => false,
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum VMError{
    #[fail(display = "Trying to push to a full stack")]
    StackOverflow,
    #[fail(display = "Trying to pop from an empty stack")]
    EmptyStack,
    #[fail(display = "Expected two numbers")]
    ExpectedNumbers,
    #[fail(display = "Expected two Strings")]
    ExpectedStrings,
    #[fail(display = "Invalid constant index {}", 0)]
    InvalidConstant(usize),
}

pub struct VM {
    allocator: Allocator,
    memory: Memory,
    ip: u64,
    sp: usize,
    stack: [Value; STACK_MAX],
    constants: Vec<Value>,
    rom: Vec<u8>,
}

impl VM {
    fn pop(&mut self) -> Result<Value, VMError> {
        if self.sp == 0 {
            return Err(VMError::EmptyStack);
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }
    fn push(&mut self, v: Value) -> Result<(), VMError> {
        if self.sp == self.stack.len() {
            return Err(VMError::StackOverflow);
        }
        self.stack[self.sp] = v;
        self.sp += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::allocator::Allocator;
    use crate::memory::Memory;
    use super::{STACK_MAX, Value, VM, VMError};

    #[test]
    fn test_pop() -> Result<(), VMError> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        let v = vm.pop()?;
        assert_eq!(v, Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_pop_on_empty_stack() {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_push() -> Result<(), VMError> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.push(Value::Integer(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_push_on_stack() {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: STACK_MAX,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        let v = vm.push(Value::Integer(1));
        assert_eq!(v, Err(VMError::StackOverflow));
    }
}

macro_rules! comp_operation {
    ($self: ident, $op: tt) => {
        match ($self.pop()?, $self.pop()?) {
            (Value::Integer(a), Value::Integer(b)) => $self.push(Value::Bool(a $op b)),
            (Value::Float(a), Value::Integer(b)) => $self.push(Value::Bool(a $op b as f32)),
            (Value::Integer(a), Value::Float(b)) => $self.push(Value::Bool((a as f32) $op b)),
            (Value::Float(a), Value::Float(b)) => $self.push(Value::Bool(a $op b)),
            (Value::Bool(a), Value::Bool(b)) => $self.push(Value::Bool(a $op b)),
            (Value::Bool(a), v) => $self.push(Value::Bool(a $op v.into())),
            (v, Value::Bool(a)) => $self.push(Value::Bool(a $op v.into())),
            (Value::Nil, Value::Nil) => $self.push(Value::Bool(false)),
            _ => $self.push(Value::Bool(false)),
        }?;
    }; 
}

macro_rules! math_operation {
    ($self: ident, $op: tt) => {
        match ($self.pop()?, $self.pop()?) {
            (Value::Integer(a), Value::Integer(b)) => $self.push(Value::Integer(a $op b)),
            (Value::Float(a), Value::Integer(b)) => $self.push(Value::Float(a $op b as f32)),
            (Value::Integer(a), Value::Float(b)) => $self.push(Value::Float(a as f32 $op b)),
            (Value::Float(a), Value::Float(b)) => $self.push(Value::Float(a $op b)),
            _ => Err(VMError::ExpectedNumbers),
        }?;
    }; 
}

impl VM {
    fn string_equal(&mut self) -> Result<(), Error> {
        match (self.pop()?, self.pop()?) {
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let string1: &String = self.memory.get_t(s1)?;
                    let string2: &String = self.memory.get_t(s2)?;
                    string1 == string2
                };
                self.push(Value::Bool(result))?;
                Ok(())
            },
            _ => Err(Error::from(VMError::ExpectedStrings)),
        }
    }

    fn string_concat(&mut self) -> Result<(), Error> {
        match (self.pop()?, self.pop()?) {
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let string1 = self.memory.get_string(s1, self.allocator.get_allocated_space(s1).unwrap())?;
                    let string2 = self.memory.get_string(s2, self.allocator.get_allocated_space(s2).unwrap())?;
                    let mut r = string1.as_bytes().to_vec();
                    r.extend(string2.as_bytes());
                    r
                };
                let address = self.allocator.malloc(result.len())?;
                self.memory.copy_u8_vector(&result, address);
                self.push(Value::String(address))?;
                Ok(())
            },
            _ => Err(Error::from(VMError::ExpectedStrings)),
        }
    }
}

impl Cpu<Instruction, VMError> for VM {
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), Error> {
        debug!("{}", instruction.to_string());
        match instruction {
            Instruction::Noop | Instruction::Return => {},
            Instruction::Constant(index) => {
                match self.constants.get(*index).cloned() {
                    Some(c) => self.push(c)?,
                    None => return Err(Error::from(VMError::InvalidConstant(*index))),
                };
            }
            Instruction::Plus => {
                math_operation!(self, +);
            }
            Instruction::Minus => {
                math_operation!(self, -);
            }
            Instruction::Mult => {
                math_operation!(self, *);
            }
            Instruction::Div => {
                math_operation!(self, /);
            }
            Instruction::Nil => self.push(Value::Nil)?,
            Instruction::True => self.push(Value::Bool(true))?,
            Instruction::False => self.push(Value::Bool(false))?,
            Instruction::Not => {
                let b: bool = self.pop()?.into();
                self.push(Value::Bool(!b))?;
            }
            Instruction::Equal => {
                comp_operation!(self, ==);
            }
            Instruction::NotEqual => {
                comp_operation!(self, !=);
            }
            Instruction::Greater => {
                comp_operation!(self, >);
            }
            Instruction::GreaterEqual => {
                comp_operation!(self, >=);
            }
            Instruction::Less => {
                comp_operation!(self, < );
            }
            Instruction::LessEqual => {
                comp_operation!(self, <=);
            }
            Instruction::StringEqual => self.string_equal()?,
            Instruction::StringConcat => self.string_concat()?,
        };
        Ok(())
    }

    #[inline]
    fn get_pc(&self) -> u16 {
        self.ip as _
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(3);
        let from = self.ip as usize;
        let to = min(from + 9, self.rom.len());
        for i in from..to {
            res.push(self.rom[i]);
        }
        res
    }

    #[inline]
    fn can_run(&self, _: &Instruction) -> bool {
        true
    }

    #[inline]
    fn is_done(&self) -> bool {
        self.ip >= self.rom.len() as _
    }

    #[inline]
    fn increase_pc(&mut self, steps: u8) {
        self.ip += u64::from(steps);
    }

    fn get_cycles_from_one_condition(
        &self,
        _: &Instruction,
        _: u8,
        _: u8,
    ) -> Result<u8, Error> {
        unimplemented!()
    }

    fn get_cycles_from_two_conditions(
        &self,
        _: &Instruction,
        _: u8,
        _: u8,
        _: u8,
    ) -> Result<u8, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod cpu_tests {
    use cpu::Cpu;
    use crate::allocator::Allocator;
    use crate::instruction::Instruction;
    use crate::memory::Memory;
    use failure::Error;
    use super::{STACK_MAX, Value, VM};

    #[test]
    fn test_constant() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: vec![Value::Integer(1)],
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Constant(0))?;
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(3));
        Ok(())
    }

    #[test]
    fn test_add_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(&Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_float_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_integer_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_sub_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(&Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_float_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_mult_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(&Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_float_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_div_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(&Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_float_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer_float() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(&Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_nil() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Nil)?;
        assert_eq!(vm.stack[0], Value::Nil);
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::True)?;
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::False)?;
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Not)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        vm.execute_instruction(&Instruction::Not)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_equals_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_equals_diff() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::NotEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_diff() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::NotEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_greater() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_lesser() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(-1);
        vm.execute_instruction(&Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_equals_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_greater() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_lesser() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(-1);
        vm.execute_instruction(&Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_greater() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_lesser() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(-1);
        vm.execute_instruction(&Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_same() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.execute_instruction(&Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_greater() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(&Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_equals_lesser() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[1] = Value::Integer(-1);
        vm.execute_instruction(&Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_same() -> Result<(), Error> {
        let memory = Memory::new(10);
        let s1 = String::from("42");
        let s2 = String::from("42");
        memory.copy_t(&s1, 0);
        memory.copy_t(&s2, 5);
        let mut vm = VM {
            allocator: Allocator::new(10),
            memory,
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::String(0);
        vm.stack[1] = Value::String(5);
        vm.execute_instruction(&Instruction::StringEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_diff() -> Result<(), Error> {
        let memory = Memory::new(10);
        let s1 = String::from("41");
        let s2 = String::from("42");
        memory.copy_t(&s1, 0);
        memory.copy_t(&s2, 5);
        let mut vm = VM {
            allocator: Allocator::new(10),
            memory,
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::String(0);
        vm.stack[1] = Value::String(5);
        vm.execute_instruction(&Instruction::StringEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_string_concat() -> Result<(), Error> {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let s1 = String::from("4");
        let s2 = String::from("2");
        let address1 = allocator.malloc(1)?;
        let address2 = allocator.malloc(1)?;
        memory.copy_string(&s1, address1);
        memory.copy_string(&s2, address2);
        let mut vm = VM {
            allocator,
            memory,
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
        };
        vm.stack[0] = Value::String(address2);
        vm.stack[1] = Value::String(address1);
        vm.execute_instruction(&Instruction::StringConcat)?;
        assert_eq!(vm.sp, 1);
        if let Value::String(address) = vm.stack[0] {
            let r = vm.memory.get_string(address, 2)?;
            assert_eq!(r, "42");
        } else {
            panic!("String concatenation should push a string");
        }
        Ok(())
    }
}
