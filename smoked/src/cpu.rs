use crate::allocator::Allocator;
use crate::memory::Memory;
use cpu::Cpu;
use crate::instruction::Instruction;
use failure::Error;
use log::debug;
use sc::{syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6};
use std::cmp::min;
use std::collections::HashMap;

const STACK_MAX: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(usize),
    Function {
        ip: usize,
        arity: usize,
    },
    Nil,
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
            Value::String(_) => true,
            Value::Function { .. } => true,
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
    #[fail(display = "Expected a function")]
    ExpectedFunction,
    #[fail(display = "Not enough arguments for function call")]
    NotEnoughArgumentsForFunction,
    #[fail(display = "Invalid constant index {}", 0)]
    InvalidConstant(usize),
    #[fail(display = "Unallocated address {}", 0)]
    UnallocatedAddress(usize),
    #[fail(display = "Global {} doesn't exist", 0)]
    GlobalDoesntExist(String),
}

struct Frame {
    stack_offset: usize,
    ip: usize,
}

pub struct VM {
    allocator: Allocator,
    memory: Memory,
    frames: Vec<Frame>,
    globals: HashMap<usize, Value>,
    sp: usize,
    stack: [Value; STACK_MAX],
    constants: Vec<Value>,
    rom: Vec<u8>,
}

impl VM {
    fn pop(&mut self) -> Result<Value, VMError> {
        let sp = self.sp - self.frames.last().unwrap().stack_offset;
        if sp == 0 {
            return Err(VMError::EmptyStack);
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    fn peek(&self) -> Result<Value, VMError> {
        let sp = self.sp - self.frames.last().unwrap().stack_offset;
        if sp == 0 {
            return Err(VMError::EmptyStack);
        }
        Ok(self.stack[self.sp - 1])
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
    use super::{STACK_MAX, Frame, Value, VM, VMError};
    use std::collections::HashMap;

    #[test]
    fn test_pop() -> Result<(), VMError> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            globals: HashMap::default(),
            rom: Vec::new(),
        };
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_pop_on_empty_stack_frame() {
        let mut vm = VM {
            allocator: Allocator::new(0),
            memory: Memory::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 1 }],
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            globals: HashMap::default(),
            rom: Vec::new(),
        };
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_push() -> Result<(), VMError> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.push(Value::Integer(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_push_on_stack() {
        let mut vm = VM {
            globals: HashMap::default(),
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
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
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let string1: &String = $self.memory.get_t(s1)?;
                    let string2: &String = $self.memory.get_t(s2)?;
                    string1 $op string2
                };
                $self.push(Value::Bool(result))
            },
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
    fn ip(&self) -> usize {
        self.frames.last().unwrap().ip
    }

    fn add_to_ip(&mut self, steps: usize) {
        self.frames.last_mut().unwrap().ip += steps;
    }

    fn string_concat(&mut self) -> Result<(), Error> {
        match (self.pop()?, self.pop()?) {
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let mut string1 = self.memory.get_u8_vector(s1, self.get_size(s1)?)?.to_vec();
                    let string2 = self.memory.get_u8_vector(s2, self.get_size(s2)?)?;
                    string1.extend(string2);
                    string1
                };
                let address = self.allocator.malloc(result.len())?;
                self.memory.copy_u8_vector(&result, address);
                self.push(Value::String(address))?;
                Ok(())
            },
            _ => Err(Error::from(VMError::ExpectedStrings)),
        }
    }

    fn syscall(&mut self) -> Result<(), Error> {
        let syscall_value = if let Value::Integer(a) = self.pop()? {
            a as usize
        } else {
            return Err(Error::from(VMError::ExpectedNumbers));
        };
        let arguments = if  let Value::Integer(a) = self.pop()? {
            a as u8
        } else {
            return Err(Error::from(VMError::ExpectedNumbers));
        };
        let ret = match arguments {
            0 => unsafe { syscall0(syscall_value) },
            1 => unsafe { syscall1(syscall_value, self.pop_usize()?) },
            2 => unsafe {
                syscall2(
                    syscall_value,
                    self.pop_usize()?,
                    self.pop_usize()?,
                )
            },
            3 => unsafe {
                syscall3(
                    syscall_value,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                )
            },
            4 => unsafe {
                syscall4(
                    syscall_value,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                )
            },
            5 => unsafe {
                syscall5(
                    syscall_value,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                )
            },
            6 => unsafe {
                syscall6(
                    syscall_value,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                    self.pop_usize()?,
                )
            },
            _ => unreachable!(),
        };
        self.push(Value::Integer(ret as _))?;
        Ok(())
    }

    fn get_global(&mut self, global: usize) -> Result<(), Error> {
        let value = *self.globals.get(&global)
            .ok_or_else(||
                Error::from(VMError::GlobalDoesntExist(self.get_constant_string(global).unwrap()))
            )?;
        self.push(value)?;
        Ok(())
    }

    fn set_global(&mut self, global: usize) -> Result<(), Error> {
        let value = self.pop()?;
        self.globals.insert(global, value);
        Ok(())
    }

    fn get_local(&mut self, local: usize) -> Result<(), Error> {
        self.push(self.stack[local])?;
        Ok(())
    }

    fn set_local(&mut self, local: usize) -> Result<(), Error> {
        self.stack[local] = self.peek()?;
        Ok(())
    }

    fn jmp_if_false(&mut self, offset: usize) -> Result<(), Error> {
        let jmp_cond: bool = self.pop()?.into();
        if !jmp_cond {
            self.add_to_ip(offset);
        }
        Ok(())
    }

    fn call(&mut self) -> Result<(), Error> {
        if let Value::Function { ip, arity } = self.pop()? {
            if self.sp < arity {
                return Err(Error::from(VMError::NotEnoughArgumentsForFunction));
            }
            let new_frame = Frame { ip, stack_offset: self.sp - arity };
            self.frames.push(new_frame);
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedFunction))
        }
    }

    fn get_size(&self, address: usize) -> Result<usize, Error> {
        self.allocator.get_allocated_space(address).ok_or(Error::from(VMError::UnallocatedAddress(address)))
    }

    fn pop_usize(&mut self) -> Result<usize, Error> {
        match self.pop()? {
            Value::Integer(a) => Ok(a as usize),
            Value::Float(f) => Ok(f as usize),
            Value::String(address) => {
                let size = self.get_size(address)?;
                let bs = self.memory.get_u8_vector(address, size)?;
                Ok(bs.as_ptr() as usize)
            },
            _ => Err(Error::from(VMError::ExpectedNumbers)),
        }
    }

    fn get_constant_string(&self, constant: usize) -> Result<String, Error> {
        let value = self.constants.get(constant).cloned()
            .ok_or(Error::from(VMError::InvalidConstant(constant)))?;
        if let Value::String(address) = value {
            Ok(self.memory.get_string(address, self.get_size(address)?)?.to_owned())
        } else {
            Err(Error::from(VMError::ExpectedStrings))
        }
    }
}

impl Cpu<Instruction, VMError> for VM {
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), Error> {
        debug!("{}", instruction.to_string());
        match instruction {
            Instruction::Noop => {},
            Instruction::Return => {
                self.frames.pop();
            },
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
            Instruction::StringConcat => self.string_concat()?,
            Instruction::Syscall => self.syscall()?,
            Instruction::GetGlobal(g) => self.get_global(*g)?,
            Instruction::SetGlobal(g) => self.set_global(*g)?,
            Instruction::GetLocal(g) => self.get_local(*g)?,
            Instruction::SetLocal(g) => self.set_local(*g)?,
            Instruction::JmpIfFalse(o) => self.jmp_if_false(*o)?,
            Instruction::Jmp(o) => {
                self.add_to_ip(*o);
            },
            Instruction::Loop(o) => {
                self.frames.last_mut().unwrap().ip -= *o;
            },
            Instruction::Call => self.call()?,
        };
        Ok(())
    }

    #[inline]
    fn get_pc(&self) -> u16 {
        self.ip() as _
    }

    #[inline]
    fn get_next_instruction_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(3);
        let from = self.ip();
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
        !self.frames.is_empty() || self.ip() >= self.rom.len() as _
    }

    #[inline]
    fn increase_pc(&mut self, steps: u8) {
        self.add_to_ip(steps as _);
    }

    fn get_cycles_from_one_condition(
        &self,
        _: &Instruction,
        _: u8,
        _: u8,
    ) -> Result<u8, Error> {
        unreachable!()
    }

    fn get_cycles_from_two_conditions(
        &self,
        _: &Instruction,
        _: u8,
        _: u8,
        _: u8,
    ) -> Result<u8, Error> {
        unreachable!()
    }
}

#[cfg(test)]
mod cpu_tests {
    use cpu::Cpu;
    use crate::allocator::Allocator;
    use crate::instruction::Instruction;
    use crate::memory::Memory;
    use failure::Error;
    use std::collections::HashMap;
    use super::{STACK_MAX, Frame, Value, VM};

    #[test]
    fn test_constant() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: vec![Value::Integer(1)],
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::Constant(0))?;
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::Nil)?;
        assert_eq!(vm.stack[0], Value::Nil);
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::True)?;
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::False)?;
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::String(0);
        vm.stack[1] = Value::String(5);
        vm.execute_instruction(&Instruction::Equal)?;
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::String(0);
        vm.stack[1] = Value::String(5);
        vm.execute_instruction(&Instruction::Equal)?;
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
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
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

    #[test]
    fn test_syscall() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[1] = Value::Integer(sc::nr::GETPID as _);
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(&Instruction::Syscall)?;
        assert_eq!(vm.sp, 1);
        if let Value::Integer(n) = vm.stack[0] {
            assert!(n > 0);
        } else {
            panic!("Syscall should return an integer");
        }
        Ok(())
    }

    #[test]
    fn test_set_global() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(&Instruction::SetGlobal(0))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.globals[&0], Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_get_global() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.globals.insert(0, Value::Integer(0));
        vm.execute_instruction(&Instruction::GetGlobal(0))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(0));
        Ok(())
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: GlobalDoesntExist(\"4\")")]
    fn test_get_global_not_existing() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let s1 = String::from("4");
        let address1 = allocator.malloc(1).unwrap();
        memory.copy_string(&s1, address1);
        let mut vm = VM {
            allocator,
            memory,
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: vec![Value::String(address1)],
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::GetGlobal(0)).unwrap();
    }

    #[test]
    fn test_set_local() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(&Instruction::SetLocal(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_get_local() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(&Instruction::GetLocal(0))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_jmping() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(&Instruction::JmpIfFalse(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 3);
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_not_jmping() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(&Instruction::JmpIfFalse(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 0);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::Jmp(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 3);
        Ok(())
    }

    #[test]
    fn test_loop() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 4, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::Loop(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 1);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(), Error> {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[1] = Value::Function {
            ip: 20,
            arity: 1,
        };
        vm.execute_instruction(&Instruction::Call)?;
        assert_eq!(vm.frames.last().unwrap().stack_offset, 0);
        assert_eq!(vm.frames.len(), 2);
        assert_eq!(vm.ip(), 20);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ExpectedFunction")]
    fn test_call_on_non_function() {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.execute_instruction(&Instruction::Call).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NotEnoughArgumentsForFunction")]
    fn test_call_without_enough_arguments() {
        let mut vm = VM {
            allocator: Allocator::new(0),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
        };
        vm.stack[1] = Value::Function {
            ip: 20,
            arity: 2,
        };
        vm.execute_instruction(&Instruction::Call).unwrap();
    }
}
