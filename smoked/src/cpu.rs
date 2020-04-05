use crate::allocator::Allocator;
use crate::memory::Memory;
use crate::instruction::{Instruction as VMInstruction};
use failure::Error;
use log::debug;
use sc::{syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6};
use std::cell::RefCell;
use std::collections::HashMap;

pub(crate) const STACK_MAX: usize = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(usize),
    Function {
        ip: usize,
        arity: usize,
    },
    Array {
        capacity: usize,
        address: usize,
    },
    Object {
        address: usize,
    }
}

const U64_SIZE: usize = std::mem::size_of::<u64>();
pub const VALUE_SIZE: usize = std::mem::size_of::<Value>();
pub const USIZE_SIZE: usize = std::mem::size_of::<usize>();

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
            Value::String(_) => true,
            Value::Array { .. } => true,
            Value::Function { .. } => true,
            Value::Nil => false,
            Value::Object { .. } => true,
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
    #[fail(display = "Expected an array")]
    ExpectedArray,
    #[fail(display = "Index out of range")]
    IndexOutOfRange,
    #[fail(display = "Not enough arguments for function call")]
    NotEnoughArgumentsForFunction,
    #[fail(display = "Invalid constant index {}", 0)]
    InvalidConstant(usize),
    #[fail(display = "Unallocated address {}", 0)]
    UnallocatedAddress(usize),
    #[fail(display = "Global {} doesn't exist", 0)]
    GlobalDoesntExist(String),
    #[fail(display = "Property {} not in object", 0)]
    PropertyDoesntExist(String),
}

pub(crate) struct Frame {
    stack_offset: usize,
    ip: usize,
}

pub struct VM {
    pub(crate) allocator: RefCell<Allocator>,
    pub(crate) memory: Memory,
    pub(crate) frames: Vec<Frame>,
    pub(crate) globals: HashMap<usize, Value>,
    pub(crate) sp: usize,
    pub(crate) stack: [Value; STACK_MAX],
    pub(crate) constants: Vec<Value>,
    pub(crate) rom: Vec<VMInstruction>,
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

    pub fn stack(&self) -> &[Value] {
        &self.stack[..self.sp]
    }
}

#[cfg(test)]
impl VM {
    fn test_vm(sp: usize) -> VM {
        VM {
            allocator: RefCell::new(Allocator::new(0)),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(0),
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
            sp,
        }
    }

    fn test_vm_with_mem(sp: usize, mem: usize) -> VM {
        VM {
            allocator: RefCell::new(Allocator::new(mem)),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            memory: Memory::new(mem),
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
            sp,
        }
    }

    fn test_vm_with_memory_and_allocator(sp: usize, memory: Memory, allocator: Allocator) -> VM {
        VM {
            allocator: RefCell::new(allocator),
            frames: vec![Frame { ip: 0, stack_offset: 0 }],
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            rom: Vec::new(),
            globals: HashMap::default(),
            memory,
            sp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{STACK_MAX, Value, VM, VMError};

    #[test]
    fn test_pop() -> Result<(), VMError> {
        let mut vm = VM::test_vm(1);
        let v = vm.pop()?;
        assert_eq!(v, Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_pop_on_empty_stack() {
        let mut vm = VM::test_vm(0);
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_pop_on_empty_stack_frame() {
        let mut vm = VM::test_vm(1);
        vm.frames[0].stack_offset = 1;
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_push() -> Result<(), VMError> {
        let mut vm = VM::test_vm(0);
        vm.push(Value::Integer(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_push_on_stack() {
        let mut vm = VM::test_vm(STACK_MAX);
        let v = vm.push(Value::Integer(1));
        assert_eq!(v, Err(VMError::StackOverflow));
    }
}

macro_rules! comp_operation {
    ($self: ident, $op: tt) => {
        match ($self.pop()?, $self.pop()?) {
            (Value::Integer(a), Value::Integer(b)) => $self.push(Value::Bool(b $op a)),
            (Value::Float(a), Value::Integer(b)) => $self.push(Value::Bool((b as f32) $op a)),
            (Value::Integer(a), Value::Float(b)) => $self.push(Value::Bool(b $op (a as f32))),
            (Value::Float(a), Value::Float(b)) => $self.push(Value::Bool(b $op a)),
            (Value::Bool(a), Value::Bool(b)) => $self.push(Value::Bool(b $op a)),
            (Value::Bool(a), v) => {
                let b: bool = v.into();
                $self.push(Value::Bool(b $op a))
            },
            (v, Value::Bool(a)) => $self.push(Value::Bool(a $op v.into())),
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let string1 = $self.memory.get_string(s2, $self.get_size(s2)?)?;
                    let string2 = $self.memory.get_string(s1, $self.get_size(s1)?)?;
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
            (Value::Integer(a), Value::Integer(b)) => $self.push(Value::Integer(b $op a)),
            (Value::Float(a), Value::Integer(b)) => $self.push(Value::Float(b as f32 $op a)),
            (Value::Integer(a), Value::Float(b)) => $self.push(Value::Float(b $op a as f32)),
            (Value::Float(a), Value::Float(b)) => $self.push(Value::Float(b $op a)),
            _ => Err(VMError::ExpectedNumbers),
        }?;
    }; 
}

impl VM {
    pub fn execute(&mut self) -> Result<u8, Error> {
        let ip = self.ip();
        self.increase_pc(1);
        self.execute_instruction(self.rom[ip].clone())?;
        Ok(0)
    }

    fn execute_instruction(&mut self, instruction: VMInstruction) -> Result<(), Error> {
        debug!("{}", instruction.to_string());
        match &instruction {
            VMInstruction::Noop => {},
            VMInstruction::Return => self.return_from_call()?,
            VMInstruction::Constant(index) => self.constant(*index)?,
            VMInstruction::Plus => {
                math_operation!(self, +);
            }
            VMInstruction::Minus => {
                math_operation!(self, -);
            }
            VMInstruction::Mult => {
                math_operation!(self, *);
            }
            VMInstruction::Div => {
                math_operation!(self, /);
            }
            VMInstruction::Nil => self.push(Value::Nil)?,
            VMInstruction::True => self.push(Value::Bool(true))?,
            VMInstruction::False => self.push(Value::Bool(false))?,
            VMInstruction::Not => {
                let b: bool = self.pop()?.into();
                self.push(Value::Bool(!b))?;
            }
            VMInstruction::Equal => {
                comp_operation!(self, ==);
            }
            VMInstruction::NotEqual => {
                comp_operation!(self, !=);
            }
            VMInstruction::Greater => {
                comp_operation!(self, >);
            }
            VMInstruction::GreaterEqual => {
                comp_operation!(self, >=);
            }
            VMInstruction::Less => {
                comp_operation!(self, < );
            }
            VMInstruction::LessEqual => {
                comp_operation!(self, <=);
            }
            VMInstruction::StringConcat => self.string_concat()?,
            VMInstruction::Syscall => self.syscall()?,
            VMInstruction::GetGlobal(g) => self.get_global(*g)?,
            VMInstruction::SetGlobal(g) => self.set_global(*g)?,
            VMInstruction::GetLocal(g) => self.get_local(*g)?,
            VMInstruction::SetLocal(g) => self.set_local(*g)?,
            VMInstruction::JmpIfFalse(o) => self.jmp_if_false(*o)?,
            VMInstruction::Jmp(o) => {
                self.add_to_ip(*o);
            },
            VMInstruction::Loop(o) => {
                self.frames.last_mut().unwrap().ip -= *o;
            },
            VMInstruction::Call => self.call()?,
            VMInstruction::ArrayAlloc => self.array_alloc()?,
            VMInstruction::ArrayGet => self.array_get()?,
            VMInstruction::ArraySet => self.array_set()?,
            VMInstruction::ObjectAlloc => self.object_alloc()?,
            VMInstruction::ObjectGet => self.object_get()?,
            VMInstruction::ObjectSet => self.object_set()?,
        };
        Ok(())
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.frames.is_empty() || self.ip() >= self.rom.len() as _
    }

    #[inline]
    fn increase_pc(&mut self, steps: u8) {
        self.add_to_ip(steps as _);
    }

    #[inline]
    fn ip(&self) -> usize {
        self.frames.last().unwrap().ip
    }

    #[inline]
    fn add_to_ip(&mut self, steps: usize) {
        self.frames.last_mut().unwrap().ip += steps;
    }

    #[inline]
    fn constant(&mut self, index: usize) -> Result<(), Error> {
        match self.constants.get(index).cloned() {
            Some(c) => self.push(c)?,
            None => return Err(Error::from(VMError::InvalidConstant(index))),
        };
        Ok(())
    }

    #[inline]
    fn return_from_call(&mut self) -> Result<(), Error> {
        let return_value = self.pop()?;
        self.sp = self.frames.last().unwrap().stack_offset;
        self.push(return_value)?;
        self.frames.pop();
        Ok(())
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
                let address = self.allocator.borrow_mut().malloc(result.len(), self.get_roots())?;
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
        self.push(self.stack[self.frames.last().unwrap().stack_offset + local])?;
        Ok(())
    }

    fn set_local(&mut self, local: usize) -> Result<(), Error> {
        self.stack[self.frames.last().unwrap().stack_offset + local] = self.peek()?;
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
            self.new_frame(ip, arity);
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedFunction))
        }
    }

    fn array_alloc(&mut self) -> Result<(), Error> {
        if let Value::Integer(capacity) = self.pop()? {
            let address = self.allocator.borrow_mut().malloc(VALUE_SIZE * capacity as usize, self.get_roots())?;
            self.push(Value::Array { capacity: capacity as usize, address })?;
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedNumbers))
        }
    }

    fn array_get(&mut self) -> Result<(), Error> {
        match (self.pop()?, self.pop()?) {
            (Value::Array { capacity, .. }, Value::Integer(index)) if capacity <= index as usize =>
                Err(Error::from(VMError::IndexOutOfRange)),
            (Value::Array { address, .. }, Value::Integer(index)) => {
                let v = self.memory.get_t::<Value>(address + index as usize * VALUE_SIZE)?.clone();
                self.push(v)?;
                Ok(())
            },
            (Value::Array { .. }, _) => Err(Error::from(VMError::ExpectedNumbers)),
            (_, _) => Err(Error::from(VMError::ExpectedArray)),
        }
    }

    fn array_set(&mut self) -> Result<(), Error> {
        match (self.pop()?, self.pop()?) {
            (Value::Array { capacity, .. }, Value::Integer(index)) if capacity <= index as usize =>
                Err(Error::from(VMError::IndexOutOfRange)),
            (Value::Array { address, .. }, Value::Integer(index)) => {
                let v = self.peek()?;
                self.memory.copy_t::<Value>(&v, address + index as usize * VALUE_SIZE);
                Ok(())
            },
            (Value::Array { .. }, _) => Err(Error::from(VMError::ExpectedNumbers)),
            (_, _) => Err(Error::from(VMError::ExpectedArray)),
        }
    }

    fn object_alloc(&mut self) -> Result<(), Error> {
        if let Value::Integer(capacity) = self.pop()? {
            let size = (VALUE_SIZE + USIZE_SIZE) * capacity as usize + USIZE_SIZE;
            let address = self.allocator.borrow_mut().malloc(size, self.get_roots())?;
            self.push(Value::Object { address })?;
            self.memory.copy_t(&0usize, address);
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedNumbers))
        }
    }

    fn object_get(&mut self) -> Result<(), Error> {
        if let (Value::Object { address: obj_address, }, Value::String(address)) = (self.pop()?, self.pop()?) {
            let size = self.allocator.borrow().get_allocated_space(address).unwrap();
            let property = self.memory.get_string(address, size)?;
            let object_length: usize = *self.memory.get_t(obj_address)?;
            let pair_bytes = self.memory
                .get_u8_vector(obj_address + USIZE_SIZE, object_length * (VALUE_SIZE + USIZE_SIZE) * U64_SIZE).unwrap();
            let bytes = unsafe {
                std::slice::from_raw_parts(pair_bytes.as_ptr() as *const (usize, Value), object_length)
            };
            let i = self.property_lookup(bytes, property)
                .map_err(|_| VMError::PropertyDoesntExist(property.to_owned()))?;
            self.push(bytes[i].1)?;
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedStrings))
        }
    }

    fn object_set(&mut self) -> Result<(), Error> {
        if let (Value::Object { address: mut obj_address, }, Value::String(address)) = (self.pop()?, self.pop()?) {
            let value = self.pop()?;
            let capacity = (self.get_size(obj_address)? - USIZE_SIZE) / (VALUE_SIZE + USIZE_SIZE);
            let object_length: usize = *self.memory.get_t(obj_address)?;
            let size = self.allocator.borrow().get_allocated_space(address).unwrap();
            let property = self.memory.get_string(address, size)?;
            let pair_bytes = self.memory
                .get_u8_vector(obj_address + USIZE_SIZE, object_length * (VALUE_SIZE + USIZE_SIZE) * U64_SIZE).unwrap();
            let bytes = unsafe {
                std::slice::from_raw_parts(pair_bytes.as_ptr() as *const (usize, Value), object_length)
            };
            let index = match self.property_lookup(bytes, property) {
                Ok(index) => {
                    index
                },
                Err(index) => {
                    if capacity <= object_length {
                        self.allocator.borrow_mut().free(obj_address)?;
                        obj_address = self.allocator.borrow_mut().malloc(
                            USIZE_SIZE + capacity * 2 * (VALUE_SIZE + USIZE_SIZE),
                            self.get_roots(),
                        )?;
                        self.memory.copy_t(&(object_length + 1), obj_address);
                        self.memory.copy_u8_vector(pair_bytes, obj_address + USIZE_SIZE);
                    }
                    for i in (index..bytes.len()).rev() {
                        self.memory.copy_t(
                            &bytes[i],
                            obj_address + USIZE_SIZE + (i+1) * (VALUE_SIZE + USIZE_SIZE)
                        );
                    }
                    self.memory.copy_t(&(object_length + 1), obj_address);
                    self.memory.copy_t(&address, obj_address + USIZE_SIZE + index * (VALUE_SIZE + USIZE_SIZE));
                    index
                }
            };
            self.memory.copy_t(&value, obj_address + USIZE_SIZE * 2 + index * (VALUE_SIZE + USIZE_SIZE));
            self.push(value)?;
            self.push(Value::Object { address: obj_address })?;
            Ok(())
        } else {
            Err(Error::from(VMError::ExpectedStrings))
        }
    }

    fn property_lookup(&self, bytes: &[(usize, Value)], property: &str) -> Result<usize, usize> {
        bytes.binary_search_by(|(curr_address, _)| {
            let found_length = self.allocator.borrow().get_allocated_space(*curr_address).unwrap();
            let found_property = self.memory.get_string(*curr_address, found_length).unwrap();
            property.cmp(found_property)
        })
    }

    fn get_size(&self, address: usize) -> Result<usize, Error> {
        self.allocator.borrow().get_allocated_space(address).ok_or(Error::from(VMError::UnallocatedAddress(address)))
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

    fn get_roots<'a>(&'a self) -> impl Iterator<Item=usize> + 'a {
        self.stack.iter()
            .chain(self.constants.iter())
            .chain(self.globals.values())
            .filter_map(move |v| {
                match v {
                    Value::String(address) => Some(vec![*address]),
                    Value::Array { address, capacity } =>
                        Some(self.get_addresses_from_array(*address, *capacity)),
                    _ => None,
                }
            })
            .flatten()
    }

    fn get_addresses_from_object(&self, address: usize) -> Vec<usize> {
        let length: usize = *self.memory.get_t(address).unwrap();
        let mut result = vec![address];
        let pair_bytes = self.memory
            .get_u8_vector(address + USIZE_SIZE, length * (VALUE_SIZE + USIZE_SIZE)).unwrap();
        let bytes = unsafe {
            std::slice::from_raw_parts(pair_bytes.as_ptr() as *const (usize, Value), length)
        };
        for (string, value) in bytes {
            result.push(*string);
            self.add_used_addresses_from_value(&mut result, value);
        }
        result
    }

    fn get_addresses_from_array(&self, address: usize, capacity: usize) -> Vec<usize> {
        let mut result = vec![address];
        for _ in 0..capacity {
            let v = self.memory.get_t::<Value>(address + capacity * std::mem::size_of::<Value>()).unwrap();
            self.add_used_addresses_from_value(&mut result, v);
        }
        result
    }

    fn add_used_addresses_from_value(&self, result: &mut Vec<usize>, v: &Value) {
        match v {
            Value::Array { address, capacity } =>
                result.extend(self.get_addresses_from_array(*address, *capacity)),
            Value::String(a) => result.push(*a),
            Value::Object { address } =>
                result.extend(self.get_addresses_from_object(*address)),
            _ => {},
        }
    }

    pub(crate) fn new_frame(&mut self, ip: usize, arity: usize) {
        let new_frame = Frame { ip, stack_offset: self.sp - arity };
        self.frames.push(new_frame);
    }
}

#[cfg(test)]
mod cpu_tests {
    use crate::allocator::Allocator;
    use crate::instruction::Instruction;
    use crate::memory::Memory;
    use failure::Error;
    use super::{Value, VM};
    use crate::cpu::{VALUE_SIZE, USIZE_SIZE};

    #[test]
    fn test_constant() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.constants.push(Value::Integer(1));
        vm.execute_instruction(Instruction::Constant(0))?;
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(3));
        Ok(())
    }

    #[test]
    fn test_add_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Plus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_sub_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Float(2.0);
        vm.execute_instruction(Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Minus)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_mult_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(Instruction::Mult)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_div_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Float(2.0);
        vm.execute_instruction(Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(Instruction::Div)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_nil() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(Instruction::Nil)?;
        assert_eq!(vm.stack[0], Value::Nil);
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(Instruction::True)?;
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(Instruction::False)?;
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.execute_instruction(Instruction::Not)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        vm.execute_instruction(Instruction::Not)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::NotEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(Instruction::NotEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_same() -> Result<(), Error> {
        let memory = Memory::new(10);
        let mut allocator = Allocator::new(10);
        let address1 = allocator.malloc(2, std::iter::empty())?;
        let address2 = allocator.malloc(2, std::iter::empty())?;
        memory.copy_string("42", address1);
        memory.copy_string("42", address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address1);
        vm.stack[1] = Value::String(address2);
        vm.execute_instruction(Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_diff() -> Result<(), Error> {
        let memory = Memory::new(10);
        let mut allocator = Allocator::new(10);
        let address1 = allocator.malloc(2, std::iter::empty())?;
        let address2 = allocator.malloc(2, std::iter::empty())?;
        memory.copy_string("41", address1);
        memory.copy_string("42", address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address1);
        vm.stack[1] = Value::String(address2);
        vm.execute_instruction(Instruction::Equal)?;
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
        let address1 = allocator.malloc(1, std::iter::empty())?;
        let address2 = allocator.malloc(1, std::iter::empty())?;
        memory.copy_string(&s1, address1);
        memory.copy_string(&s2, address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address2);
        vm.stack[1] = Value::String(address1);
        vm.execute_instruction(Instruction::StringConcat)?;
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
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(sc::nr::GETPID as _);
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(Instruction::Syscall)?;
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
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(Instruction::SetGlobal(0))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.globals[&0], Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_get_global() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.globals.insert(0, Value::Integer(0));
        vm.execute_instruction(Instruction::GetGlobal(0))?;
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
        let address1 = allocator.malloc(1, std::iter::empty()).unwrap();
        memory.copy_string(&s1, address1);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.constants = vec![Value::String(address1)];
        vm.execute_instruction(Instruction::GetGlobal(0)).unwrap();
    }

    #[test]
    fn test_set_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::SetLocal(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_get_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::GetLocal(0))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(Instruction::JmpIfFalse(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 3);
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_not_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::JmpIfFalse(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 0);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(Instruction::Jmp(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 3);
        Ok(())
    }

    #[test]
    fn test_loop() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.frames[0].ip = 4;
        vm.execute_instruction(Instruction::Loop(3))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 1);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Function {
            ip: 20,
            arity: 1,
        };
        vm.execute_instruction(Instruction::Call)?;
        assert_eq!(vm.frames.last().unwrap().stack_offset, 0);
        assert_eq!(vm.frames.len(), 2);
        assert_eq!(vm.ip(), 20);
        Ok(())
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ExpectedFunction")]
    fn test_call_on_non_function() {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(Instruction::Call).unwrap();
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NotEnoughArgumentsForFunction")]
    fn test_call_without_enough_arguments() {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Function {
            ip: 20,
            arity: 2,
        };
        vm.execute_instruction(Instruction::Call).unwrap();
    }

    #[test]
    fn test_array_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::ArrayAlloc).unwrap();
        if let Value::Array { capacity, address } = vm.stack[0] {
            assert_eq!(vm.allocator.borrow().get_allocated_space(address).unwrap(), capacity * VALUE_SIZE);
        } else {
            panic!("Expected array as output of ArrayAlloc {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_array_get() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let value = Value::Integer(42);
        let address = allocator.malloc(std::mem::size_of::<Value>(), std::iter::empty()).unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::Integer(0);
        vm.stack[1] = Value::Array { address, capacity: 1};
        vm.execute_instruction(Instruction::ArrayGet).unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(42));
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: IndexOutOfRange")]
    fn test_array_get_out_of_range() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let value = Value::Integer(42);
        let address = allocator.malloc(std::mem::size_of::<Value>(), std::iter::empty()).unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Array { address, capacity: 1};
        vm.execute_instruction(Instruction::ArrayGet).unwrap();
    }

    #[test]
    fn test_array_set() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let value = Value::Integer(42);
        let address = allocator.malloc(std::mem::size_of::<Value>(), std::iter::empty()).unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = Value::Integer(0);
        vm.stack[2] = Value::Array { address, capacity: 1};
        vm.execute_instruction(Instruction::ArraySet).unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.memory.get_t::<Value>(address).unwrap().clone(), Value::Integer(0));
        assert_eq!(vm.stack[0], Value::Integer(0));
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: IndexOutOfRange")]
    fn test_array_set_out_of_range() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let value = Value::Integer(42);
        let address = allocator.malloc(std::mem::size_of::<Value>(), std::iter::empty()).unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = Value::Integer(1);
        vm.stack[2] = Value::Array { address, capacity: 1};
        vm.execute_instruction(Instruction::ArraySet).unwrap();
    }

    #[test]
    fn test_object_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(Instruction::ObjectAlloc).unwrap();
        if let Value::Object { address } = vm.stack[0] {
            assert_eq!(
                0usize,
                *vm.memory.get_t(address).unwrap(),
            );
            assert_eq!(
                vm.allocator.borrow().get_allocated_space(address).unwrap(),
                VALUE_SIZE + USIZE_SIZE * 2,
            );
        } else {
            panic!("Expected array as output of ArrayAlloc {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_object_get() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator.malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address);
        vm.stack[1] = Value::Object { address: obj_address };
        vm.execute_instruction(Instruction::ObjectGet).unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(42));
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: PropertyDoesntExist(\"VALUE1\")")]
    fn test_object_get_wrong_key() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let wrong_address = allocator.malloc(6, std::iter::empty()).unwrap();
        memory.copy_string("VALUE1", wrong_address);
        let obj_address = allocator.malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(wrong_address);
        vm.stack[1] = Value::Object { address: obj_address };
        vm.execute_instruction(Instruction::ObjectGet).unwrap();
    }

    #[test]
    fn test_object_set() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator.malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&0usize, obj_address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address);
        vm.stack[2] = Value::Object { address: obj_address };
        vm.execute_instruction(Instruction::ObjectSet).unwrap();
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm.memory.get_t::<Value>(obj_address + USIZE_SIZE * 2).unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(address_got, address);
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        assert_eq!(vm.stack[1], Value::Object { address: obj_address });
    }

    #[test]
    fn test_object_set_on_existing() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator.malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address);
        vm.stack[2] = Value::Object { address: obj_address };
        vm.execute_instruction(Instruction::ObjectSet).unwrap();
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm.memory.get_t::<Value>(obj_address + USIZE_SIZE * 2).unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(address_got, address);
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        assert_eq!(vm.stack[1], Value::Object { address: obj_address });
    }

    #[test]
    fn test_object_set_on_non_existing_without_space() {
        let memory = Memory::new(100);
        let mut allocator = Allocator::new(100);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let address2 = allocator.malloc(6, std::iter::empty()).unwrap();
        memory.copy_string("VALUE1", address2);
        let obj_address = allocator.malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address2);
        vm.stack[2] = Value::Object { address: obj_address };
        vm.execute_instruction(Instruction::ObjectSet).unwrap();
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        if let Value::Object { address: obj_address } = &vm.stack[1] {
            let obj_address = *obj_address;
            let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
            let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
            let value_got = vm.memory.get_t::<Value>(obj_address + USIZE_SIZE * 2).unwrap();
            let address_got2 = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE * 2 + VALUE_SIZE).unwrap();
            let value_got2 = vm.memory.get_t::<Value>(obj_address + USIZE_SIZE * 3 + VALUE_SIZE).unwrap();
            assert_eq!(length_got, 2);
            assert_eq!(address_got, address2);
            assert_eq!(address_got2, address);
            assert_eq!(value_got, &Value::Integer(42));
            assert_eq!(value_got2, &Value::Integer(41));
        }
    }
}
