use crate::allocator::Allocator;
use crate::instruction::{Instruction, InstructionType};
use crate::memory::Memory;
use failure::Error;
use failure::_core::fmt::Formatter;
use sc::{syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;

pub(crate) const STACK_MAX: usize = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Nil,
    Integer(i64),
    Float(f32),
    Bool(bool),
    String(usize),
    Pointer(usize),
    Function { ip: usize, arity: usize, uplifts: Option<usize> },
    Array { capacity: usize, address: usize },
    Object { address: usize },
}

impl Into<Vec<u8>> for Value {
    fn into(self) -> Vec<u8> {
        let mut ret = vec![];
        match self {
            Value::Nil => ret.push(0),
            Value::Integer(i) => {
                ret.push(1);
                ret.extend_from_slice(&i.to_le_bytes());
            }
            Value::Float(f) => {
                ret.push(2);
                ret.extend_from_slice(&f.to_le_bytes());
            }
            Value::Bool(b) => {
                ret.push(3);
                ret.push(if b { 1u8 } else { 0u8 });
            }
            Value::String(s) => {
                ret.push(4);
                ret.extend_from_slice(&s.to_le_bytes());
            }
            Value::Function { ip, arity, uplifts } => {
                ret.push(5);
                ret.extend_from_slice(&ip.to_le_bytes());
                ret.extend_from_slice(&arity.to_le_bytes());
                if let Some(uplifts) = uplifts {
                    ret.push(1);
                    ret.extend_from_slice(&uplifts.to_le_bytes());
                } else {
                    ret.push(0);
                }
            }
            Value::Array { capacity, .. } => {
                ret.push(6);
                ret.extend_from_slice(&capacity.to_le_bytes());
            }
            Value::Object { .. } => {
                ret.push(7);
                ret.extend_from_slice(&0usize.to_le_bytes());
            }
            Value::Pointer(address) => {
                ret.push(8);
                ret.extend_from_slice(&address.to_le_bytes())
            }
        }
        ret
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
            Value::Pointer(_) => true,
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum VMErrorType {
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

#[derive(Debug, Fail, PartialEq)]
pub struct VMError {
    error_type: VMErrorType,
    file: String,
    line: usize,
}

impl Display for VMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("[{} line {}] {}", self.file, self.line, self.error_type).as_str())
    }
}

pub(crate) struct Frame {
    arity: usize,
    ip: usize,
    stack_offset: usize,
}

#[derive(Debug, PartialEq)]
pub struct Location {
    pub address: usize,
    pub line: usize,
}

pub struct VM {
    pub(crate) allocator: RefCell<Allocator>,
    pub(crate) memory: Memory,
    pub(crate) frames: Vec<Frame>,
    pub(crate) globals: HashMap<usize, Value>,
    pub(crate) sp: usize,
    pub(crate) stack: [Value; STACK_MAX],
    pub constants: Vec<Value>,
    pub rom: Vec<Instruction>,
    pub locations: Vec<Location>,
}

impl VM {
    pub fn new(
        allocator: Allocator,
        constants: Vec<Value>,
        locations: Vec<Location>,
        memory: Memory,
        rom: Vec<Instruction>,
    ) -> VM {
        VM {
            allocator: RefCell::new(allocator),
            frames: vec![],
            globals: HashMap::new(),
            sp: 0,
            stack: [Value::Nil; STACK_MAX],
            constants,
            locations,
            memory,
            rom,
        }
    }

    fn pop(&mut self) -> Result<Value, Error> {
        if (self.sp - self.frames.last().unwrap().stack_offset) == 0 {
            Err(self.create_error(VMErrorType::EmptyStack)?)?;
        }
        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    fn peek(&self) -> Result<Value, Error> {
        if (self.sp - self.frames.last().unwrap().stack_offset) == 0 {
            Err(self.create_error(VMErrorType::EmptyStack)?)?;
        }
        Ok(self.stack[self.sp - 1])
    }

    fn push(&mut self, v: Value) -> Result<(), Error> {
        if self.sp == self.stack.len() {
            Err(self.create_error(VMErrorType::StackOverflow)?)?;
        }
        self.stack[self.sp] = v;
        self.sp += 1;
        Ok(())
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack[..self.sp]
    }

    fn create_error(&self, error_type: VMErrorType) -> Result<VMError, Error> {
        let location = self.rom[self.ip() - 1].location;
        let file = self
            .memory
            .get_string(
                self.locations[location].address,
                self.get_size(self.locations[location].address)?,
            )?
            .to_owned();
        Ok(VMError {
            line: self.locations[location].line,
            error_type,
            file,
        })
    }

    fn dereference_pointer(&self, value: Value) -> Result<Value, Error> {
        if let Value::Pointer(address) = value {
            Ok(self.memory.get_t::<Value>(address)?.clone())
        } else {
            Ok(value)
        }
    }
    fn dereference_pop(&mut self) -> Result<Value, Error> {
        let value = self.pop()?;
        self.dereference_pointer(value)
    }
}

#[cfg(test)]
impl VM {
    fn test_vm(sp: usize) -> VM {
        let allocator = RefCell::new(Allocator::new(10));
        allocator
            .borrow_mut()
            .malloc(4, std::iter::empty())
            .unwrap();
        let memory = Memory::new(10);
        memory.copy_string("hola", 0);
        VM {
            constants: Vec::new(),
            frames: vec![Frame {
                arity: 0,
                ip: 1,
                stack_offset: 0,
            }],
            globals: HashMap::default(),
            locations: vec![Location {
                address: 0,
                line: 0,
            }],
            stack: [Value::Integer(0); STACK_MAX],
            rom: vec![Instruction {
                instruction_type: InstructionType::Noop,
                location: 0,
            }],
            allocator,
            memory,
            sp,
        }
    }

    fn test_vm_with_mem(sp: usize, mem: usize) -> VM {
        VM {
            allocator: RefCell::new(Allocator::new(mem)),
            constants: Vec::new(),
            frames: vec![Frame {
                arity: 0,
                ip: 0,
                stack_offset: 0,
            }],
            globals: HashMap::default(),
            locations: vec![],
            memory: Memory::new(mem),
            stack: [Value::Integer(0); STACK_MAX],
            rom: Vec::new(),
            sp,
        }
    }

    fn test_vm_with_memory_and_allocator(sp: usize, memory: Memory, allocator: Allocator) -> VM {
        let allocator = RefCell::new(allocator);
        let address = allocator
            .borrow_mut()
            .malloc(4, std::iter::empty())
            .unwrap();
        memory.copy_string("hola", address);
        VM {
            constants: Vec::new(),
            frames: vec![Frame {
                arity: 0,
                ip: 1,
                stack_offset: 0,
            }],
            globals: HashMap::default(),
            locations: vec![Location { address, line: 0 }],
            rom: vec![Instruction {
                instruction_type: InstructionType::Noop,
                location: 0,
            }],
            stack: [Value::Integer(0); STACK_MAX],
            allocator,
            memory,
            sp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Value, STACK_MAX, VM};
    use failure::Error;

    #[test]
    fn test_pop() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        let v = vm.pop()?;
        assert_eq!(v, Value::Integer(0));
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: EmptyStack, file: \"hola\", line: 0 }"
    )]
    fn test_pop_on_empty_stack() {
        let mut vm = VM::test_vm(0);
        vm.pop().unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: EmptyStack, file: \"hola\", line: 0 }"
    )]
    fn test_pop_on_empty_stack_frame() {
        let mut vm = VM::test_vm(1);
        vm.frames[0].stack_offset = 1;
        vm.pop().unwrap();
    }

    #[test]
    fn test_push() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.push(Value::Integer(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: StackOverflow, file: \"hola\", line: 0 }"
    )]
    fn test_push_on_stack() {
        let mut vm = VM::test_vm(STACK_MAX);
        vm.push(Value::Integer(1)).unwrap();
    }
}

macro_rules! comp_operation {
    ($self: ident, $op: tt) => {
        match ($self.dereference_pop()?, $self.dereference_pop()?) {
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

macro_rules! logical_operation {
    ($self: ident, $op: tt) => {
        let value_a = $self.dereference_pop()?;
        let value_b = $self.dereference_pop()?;
        let a: bool = value_a.into();
        let b: bool = value_b.into();
        $self.push(Value::Bool(b $op a))?;
    };
}

macro_rules! math_operation {
    ($self: ident, $op: tt, $location: expr) => {
        match ($self.dereference_pop()?, $self.dereference_pop()?) {
            (Value::Integer(a), Value::Integer(b)) => $self.push(Value::Integer(b $op a)),
            (Value::Float(a), Value::Integer(b)) => $self.push(Value::Float(b as f32 $op a)),
            (Value::Integer(a), Value::Float(b)) => $self.push(Value::Float(b $op a as f32)),
            (Value::Float(a), Value::Float(b)) => $self.push(Value::Float(b $op a)),
            _ => Err(Error::from($self.create_error(VMErrorType::ExpectedNumbers)?)),
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

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), Error> {
        match &instruction.instruction_type {
            InstructionType::Noop => {}
            InstructionType::Return => self.return_from_call()?,
            InstructionType::Constant(index) => self.constant(*index)?,
            InstructionType::Plus => {
                math_operation!(self, +, instruction.location);
            }
            InstructionType::Minus => {
                math_operation!(self, -, instruction.location);
            }
            InstructionType::Mult => {
                math_operation!(self, *, instruction.location);
            }
            InstructionType::Div => {
                math_operation!(self, /, instruction.location);
            }
            InstructionType::Nil => self.push(Value::Nil)?,
            InstructionType::True => self.push(Value::Bool(true))?,
            InstructionType::False => self.push(Value::Bool(false))?,
            InstructionType::Not => {
                let b: bool = self.dereference_pop()?.into();
                self.push(Value::Bool(!b))?;
            }
            InstructionType::Equal => {
                comp_operation!(self, ==);
            }
            InstructionType::NotEqual => {
                comp_operation!(self, !=);
            }
            InstructionType::Greater => {
                comp_operation!(self, >);
            }
            InstructionType::GreaterEqual => {
                comp_operation!(self, >=);
            }
            InstructionType::Less => {
                comp_operation!(self, < );
            }
            InstructionType::LessEqual => {
                comp_operation!(self, <=);
            }
            InstructionType::And => {
                logical_operation!(self, &&);
            }
            InstructionType::Or => {
                logical_operation!(self, ||);
            }
            InstructionType::Abs => {
                let v = self.dereference_pop()?;
                match v {
                    Value::Integer(a) => self.push(Value::Integer(a.abs()))?,
                    Value::Float(a) => self.push(Value::Float(a.abs()))?,
                    _ => Err(self.create_error(VMErrorType::ExpectedNumbers)?)?,
                };
            }
            InstructionType::StringConcat => self.string_concat()?,
            InstructionType::Syscall => self.syscall()?,
            InstructionType::GetGlobal(g) => self.get_global(*g)?,
            InstructionType::SetGlobal(g) => self.set_global(*g)?,
            InstructionType::GetLocal(g) => self.get_local(*g)?,
            InstructionType::SetLocal(g) => self.set_local(*g)?,
            InstructionType::JmpIfFalse(o) => self.jmp_if_false(*o)?,
            InstructionType::Jmp(o) => {
                self.add_to_ip(*o);
            }
            InstructionType::Loop(o) => {
                self.frames.last_mut().unwrap().ip -= *o;
            }
            InstructionType::Call => self.call()?,
            InstructionType::ArrayAlloc => self.array_alloc()?,
            InstructionType::ArrayGet => self.array_get()?,
            InstructionType::ArraySet => self.array_set()?,
            InstructionType::MultiArraySet => self.multi_array_set()?,
            InstructionType::ObjectAlloc => self.object_alloc()?,
            InstructionType::ObjectGet => self.object_get()?,
            InstructionType::ObjectSet => self.object_set()?,
            InstructionType::Pop => {
                self.pop()?;
            },
            InstructionType::Push => self.push(self.peek()?)?,
            InstructionType::RepeatedArraySet => self.repeated_array_set()?,
            InstructionType::Strlen => self.strlen()?,
            InstructionType::Swap => self.swap()?,
            InstructionType::ToStr => self.instr_to_str()?,
            InstructionType::Uplift(local) => self.uplift(*local)?,
            InstructionType::AttachArray(function) => self.attach_array(*function)?,
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
            None => {
                Err(self.create_error(VMErrorType::InvalidConstant(index))?)?;
            }
        };
        Ok(())
    }

    #[inline]
    fn return_from_call(&mut self) -> Result<(), Error> {
        let return_value = {
            let previous_frame = self.frames.last().unwrap();
            let pso = previous_frame.stack_offset;
            let r = if self.sp - previous_frame.arity > previous_frame.stack_offset {
                Some(self.pop()?)
            } else {
                None
            };
            self.sp = pso;
            r
        };
        if let Some(return_value) = return_value {
            self.push(return_value)?;
        }
        self.frames.pop();
        Ok(())
    }

    fn string_concat(&mut self) -> Result<(), Error> {
        match (self.dereference_pop()?, self.dereference_pop()?) {
            (Value::String(s1), Value::String(s2)) => {
                let result = {
                    let mut string1 = self.memory.get_u8_vector(s1, self.get_size(s1)?)?.to_vec();
                    let string2 = self.memory.get_u8_vector(s2, self.get_size(s2)?)?;
                    string1.extend(string2);
                    string1
                };
                let address = self
                    .allocator
                    .borrow_mut()
                    .malloc(result.len(), self.get_roots())?;
                self.memory.copy_u8_vector(&result, address);
                self.push(Value::String(address))?;
            }
            _ => Err(self.create_error(VMErrorType::ExpectedStrings)?)?,
        };
        Ok(())
    }

    fn syscall(&mut self) -> Result<(), Error> {
        let syscall_value = self.pop_usize()?;
        let arguments = self.pop_usize()?;
        let ret = match arguments {
            0 => unsafe { syscall0(syscall_value) },
            1 => unsafe { syscall1(syscall_value, self.pop_usize()?) },
            2 => unsafe { syscall2(syscall_value, self.pop_usize()?, self.pop_usize()?) },
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
        match self.globals.get(&global).cloned() {
            None => {
                Err(self.create_error(VMErrorType::GlobalDoesntExist(
                    self.get_constant_string(global).unwrap(),
                ))?)?;
            }
            Some(value) => self.push(value)?,
        };
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

    fn uplift(&mut self, local: usize) -> Result<(), Error> {
        let value = self.stack[self.frames.last().unwrap().stack_offset + local];
        let address = self.allocator.borrow_mut().malloc_t::<Value, _>(self.get_roots())?;
        self.memory.copy_t(&value, address);
        self.stack[self.frames.last().unwrap().stack_offset + local] = Value::Pointer(address);
        self.push(Value::Pointer(address))?;
        Ok(())
    }

    fn attach_array(&mut self, global: usize) -> Result<(), Error> {
        let function = self.globals.get(&global).cloned();
        if let None = function {
            return Err(Error::from(self.create_error(VMErrorType::InvalidConstant(global))?));
        }
        if let Some(Value::Function { ip, arity, .. }) = function {
            let address = if let Value::Array { address, .. } = self.pop()? {
                address
            } else {
                return Err(Error::from(self.create_error(VMErrorType::ExpectedArray)?));
            };
            self.globals.insert(global, Value::Function { ip, arity, uplifts: Some(address) });
            Ok(())
        } else {
            Err(Error::from(self.create_error(VMErrorType::ExpectedFunction)?))
        }
    }

    fn jmp_if_false(&mut self, offset: usize) -> Result<(), Error> {
        let jmp_cond: bool = self.dereference_pop()?.into();
        if !jmp_cond {
            self.add_to_ip(offset);
        }
        Ok(())
    }

    fn call(&mut self) -> Result<(), Error> {
        if let Value::Function { ip, arity, uplifts } = self.pop()? {
            if self.sp < arity {
                Err(self.create_error(VMErrorType::NotEnoughArgumentsForFunction)?)?;
            }
            self.new_frame(ip, arity);
            if let Some(address) = uplifts {
                let array_size = self.get_size(address)? / VALUE_SIZE;
                for i in 0..array_size {
                    let value = *self.memory.get_t::<Value>(address + i * VALUE_SIZE)?;
                    self.push(value)?;
                    self.set_local(i)?;
                }
            }
        } else {
            Err(self.create_error(VMErrorType::ExpectedNumbers)?)?;
        }
        Ok(())
    }

    fn array_alloc(&mut self) -> Result<(), Error> {
        if let Value::Integer(capacity) = self.dereference_pop()? {
            let address = self
                .allocator
                .borrow_mut()
                .malloc(VALUE_SIZE * capacity as usize, self.get_roots())?;
            self.push(Value::Array {
                capacity: capacity as usize,
                address,
            })?;
        } else {
            Err(self.create_error(VMErrorType::ExpectedNumbers)?)?;
        }
        Ok(())
    }

    fn array_get(&mut self) -> Result<(), Error> {
        match (self.dereference_pop()?, self.dereference_pop()?) {
            (Value::Array { capacity, .. }, Value::Integer(index))
                if capacity <= index as usize =>
            {
                Err(self.create_error(VMErrorType::IndexOutOfRange)?)?
            }
            (Value::Array { address, .. }, Value::Integer(index)) => {
                let v = self
                    .memory
                    .get_t::<Value>(address + index as usize * VALUE_SIZE)?
                    .clone();
                self.push(v)?;
            }
            (Value::Array { .. }, _) => Err(self.create_error(VMErrorType::ExpectedNumbers)?)?,
            (_, _) => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn array_set(&mut self) -> Result<(), Error> {
        match (self.dereference_pop()?, self.dereference_pop()?) {
            (Value::Array { capacity, .. }, Value::Integer(index))
                if capacity <= index as usize =>
            {
                Err(self.create_error(VMErrorType::IndexOutOfRange)?)?
            }
            (Value::Array { address, .. }, Value::Integer(index)) => {
                let v = self.peek()?;
                self.memory
                    .copy_t::<Value>(&v, address + index as usize * VALUE_SIZE);
            }
            (Value::Array { .. }, _) => Err(self.create_error(VMErrorType::ExpectedNumbers)?)?,
            (_, _) => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn multi_array_set(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            Value::Array { address, capacity } => {
                let mut vs = vec![];
                for _ in 0..capacity {
                    let v = self.pop()?;
                    vs.push(v);
                }
                self.memory.copy_t_slice(&vs, address);
                self.push(Value::Array { address, capacity })?;
            }
            _ => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn repeated_array_set(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            Value::Array { address, capacity } => {
                let v = self.pop()?;
                let vs = vec![v].repeat(capacity);
                self.memory.copy_t_slice(&vs, address);
                self.push(Value::Array { address, capacity })?;
            }
            _ => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn object_alloc(&mut self) -> Result<(), Error> {
        if let Value::Integer(capacity) = self.dereference_pop()? {
            let size = (VALUE_SIZE + USIZE_SIZE) * capacity as usize + USIZE_SIZE;
            let address = self.allocator.borrow_mut().malloc(size, self.get_roots())?;
            self.push(Value::Object { address })?;
            self.memory.copy_t(&0usize, address);
        } else {
            Err(self.create_error(VMErrorType::ExpectedNumbers)?)?;
        }
        Ok(())
    }

    fn object_get(&mut self) -> Result<(), Error> {
        if let (
            Value::Object {
                address: obj_address,
            },
            Value::String(address),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let size = self
                .allocator
                .borrow()
                .get_allocated_space(address)
                .unwrap();
            let property = self.memory.get_string(address, size)?;
            let object_length: usize = *self.memory.get_t(obj_address)?;
            let pair_bytes = self
                .memory
                .get_u8_vector(
                    obj_address + USIZE_SIZE,
                    object_length * (VALUE_SIZE + USIZE_SIZE) * U64_SIZE,
                )
                .unwrap();
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    pair_bytes.as_ptr() as *const (usize, Value),
                    object_length,
                )
            };
            let i = match self.property_lookup(bytes, property) {
                Ok(i) => i,
                Err(_) => {
                    Err(self.create_error(VMErrorType::PropertyDoesntExist(property.to_owned()))?)?
                }
            };
            self.push(bytes[i].1)?;
        } else {
            Err(self.create_error(VMErrorType::ExpectedStrings)?)?;
        }
        Ok(())
    }

    fn object_set(&mut self) -> Result<(), Error> {
        if let (
            Value::Object {
                address: mut obj_address,
            },
            Value::String(address),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let value = self.pop()?;
            let capacity = (self.get_size(obj_address)? - USIZE_SIZE) / (VALUE_SIZE + USIZE_SIZE);
            let object_length: usize = *self.memory.get_t(obj_address)?;
            let size = self
                .allocator
                .borrow()
                .get_allocated_space(address)
                .unwrap();
            let property = self.memory.get_string(address, size)?;
            let pair_bytes = self
                .memory
                .get_u8_vector(
                    obj_address + USIZE_SIZE,
                    object_length * (VALUE_SIZE + USIZE_SIZE) * U64_SIZE,
                )
                .unwrap();
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    pair_bytes.as_ptr() as *const (usize, Value),
                    object_length,
                )
            };
            let index = match self.property_lookup(bytes, property) {
                Ok(index) => index,
                Err(index) => {
                    if capacity <= object_length {
                        self.allocator.borrow_mut().free(obj_address)?;
                        obj_address = self.allocator.borrow_mut().malloc(
                            USIZE_SIZE + capacity * 2 * (VALUE_SIZE + USIZE_SIZE),
                            self.get_roots(),
                        )?;
                        self.memory.copy_t(&(object_length + 1), obj_address);
                        self.memory
                            .copy_u8_vector(pair_bytes, obj_address + USIZE_SIZE);
                    }
                    for i in (index..bytes.len()).rev() {
                        self.memory.copy_t(
                            &bytes[i],
                            obj_address + USIZE_SIZE + (i + 1) * (VALUE_SIZE + USIZE_SIZE),
                        );
                    }
                    self.memory.copy_t(&(object_length + 1), obj_address);
                    self.memory.copy_t(
                        &address,
                        obj_address + USIZE_SIZE + index * (VALUE_SIZE + USIZE_SIZE),
                    );
                    index
                }
            };
            self.memory.copy_t(
                &value,
                obj_address + USIZE_SIZE * 2 + index * (VALUE_SIZE + USIZE_SIZE),
            );
            self.push(value)?;
            self.push(Value::Object {
                address: obj_address,
            })?;
        } else {
            Err(self.create_error(VMErrorType::ExpectedStrings)?)?;
        }
        Ok(())
    }

    fn strlen(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            Value::String(s) => {
                let s_size = self.get_size(s)?;
                self.push(Value::Integer(s_size as _))?;
            },
            _ => Err(self.create_error(VMErrorType::ExpectedStrings)?)?,
        };
        Ok(())
    }

    fn swap(&mut self) -> Result<(), Error> {
        let botttom = self.pop()?;
        let top = self.pop()?;
        self.push(botttom)?;
        self.push(top)?;
        Ok(())
    }

    fn instr_to_str(&mut self) -> Result<(), Error> {
        let v = self.dereference_pop()?;
        if let Value::String(address) = v {
            self.push(Value::String(address))?;
        } else {
            let s = match v {
                Value::Nil => "nil".to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Float(f) => f.to_string(),
                Value::Function { .. } => "[function]".to_string(),
                Value::Array { .. } => "[array]".to_string(),
                Value::Object { .. } => "[object]".to_string(),
                _ => panic!("Cannot happen"),
            };
            let a = self.allocator.borrow_mut().malloc(s.len(), self.get_roots())?;
            self.memory.copy_u8_vector(s.as_bytes(), a);
            self.push(Value::String(a))?;
        }
        Ok(())
    }

    fn property_lookup(&self, bytes: &[(usize, Value)], property: &str) -> Result<usize, usize> {
        bytes.binary_search_by(|(curr_address, _)| {
            let found_length = self
                .allocator
                .borrow()
                .get_allocated_space(*curr_address)
                .unwrap();
            let found_property = self.memory.get_string(*curr_address, found_length).unwrap();
            property.cmp(found_property)
        })
    }

    fn get_size(&self, address: usize) -> Result<usize, Error> {
        match self.allocator.borrow().get_allocated_space(address) {
            Some(ret) => Ok(ret),
            None => Err(Error::from(
                self.create_error(VMErrorType::UnallocatedAddress(address))?,
            )),
        }
    }

    fn pop_usize(&mut self) -> Result<usize, Error> {
        let ret = match self.dereference_pop()? {
            Value::Integer(a) => a as usize,
            Value::Float(f) => f as usize,
            Value::String(address) => {
                let size = self.get_size(address)?;
                let bs = self.memory.get_u8_vector(address, size)?;
                bs.as_ptr() as usize
            }
            _ => Err(self.create_error(VMErrorType::ExpectedNumbers)?)?,
        };
        Ok(ret)
    }

    fn get_constant_string(&self, constant: usize) -> Result<String, Error> {
        let value = self.constants.get(constant).cloned().ok_or_else(|| {
            self.create_error(VMErrorType::InvalidConstant(constant))
                .unwrap()
        })?;
        let ret = match value {
            Value::String(address) => self
                .memory
                .get_string(address, self.get_size(address)?)?
                .to_owned(),
            _ => Err(self.create_error(VMErrorType::ExpectedStrings)?)?,
        };
        Ok(ret)
    }

    fn get_roots<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.stack
            .iter()
            .chain(self.constants.iter())
            .chain(self.globals.values())
            .filter_map(move |v| match v {
                Value::String(address) => Some(vec![*address]),
                Value::Array { address, capacity } => {
                    Some(self.get_addresses_from_array(*address, *capacity))
                }
                _ => None,
            })
            .flatten()
    }

    fn get_addresses_from_object(&self, address: usize) -> Vec<usize> {
        let length: usize = *self.memory.get_t(address).unwrap();
        let mut result = vec![address];
        let pair_bytes = self
            .memory
            .get_u8_vector(address + USIZE_SIZE, length * (VALUE_SIZE + USIZE_SIZE))
            .unwrap();
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
            let v = self
                .memory
                .get_t::<Value>(address + capacity * std::mem::size_of::<Value>())
                .unwrap();
            self.add_used_addresses_from_value(&mut result, v);
        }
        result
    }

    fn add_used_addresses_from_value(&self, result: &mut Vec<usize>, v: &Value) {
        match v {
            Value::Array { address, capacity } => {
                result.extend(self.get_addresses_from_array(*address, *capacity))
            }
            Value::String(a) => result.push(*a),
            Value::Object { address } => result.extend(self.get_addresses_from_object(*address)),
            _ => {}
        }
    }

    pub(crate) fn new_frame(&mut self, ip: usize, arity: usize) {
        let new_frame = Frame {
            arity: 0,
            ip,
            stack_offset: self.sp - arity,
        };
        self.frames.push(new_frame);
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::{Value, VM};
    use crate::allocator::Allocator;
    use crate::cpu::{USIZE_SIZE, VALUE_SIZE};
    use crate::instruction::{Instruction, InstructionType};
    use crate::memory::Memory;
    use failure::Error;

    fn create_instruction(instruction_type: InstructionType) -> Instruction {
        Instruction {
            instruction_type,
            location: 0,
        }
    }

    #[test]
    fn test_constant() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.constants.push(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::Constant(0)))?;
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(3));
        Ok(())
    }

    #[test]
    fn test_add_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_add_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(3.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_sub_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Float(2.0);
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_sub_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(1.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_mult_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Float(2.0);
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_mult_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Float(1.0);
        vm.stack[1] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(2));
        Ok(())
    }

    #[test]
    fn test_div_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Float(2.0);
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_div_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Float(1.0);
        vm.stack[0] = Value::Integer(2);
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Float(2.0));
        Ok(())
    }

    #[test]
    fn test_nil() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::Nil))?;
        assert_eq!(vm.stack[0], Value::Nil);
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::True))?;
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::False))?;
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.execute_instruction(create_instruction(InstructionType::Not))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        vm.execute_instruction(create_instruction(InstructionType::Not))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::NotEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::NotEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = Value::Integer(-1);
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_same() -> Result<(), Error> {
        let memory = Memory::new(20);
        let mut allocator = Allocator::new(20);
        let address1 = allocator.malloc(2, std::iter::empty())?;
        let address2 = allocator.malloc(2, std::iter::empty())?;
        memory.copy_string("42", address1);
        memory.copy_string("42", address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address1);
        vm.stack[1] = Value::String(address2);
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_equals_diff() -> Result<(), Error> {
        let memory = Memory::new(20);
        let mut allocator = Allocator::new(20);
        let address1 = allocator.malloc(2, std::iter::empty())?;
        let address2 = allocator.malloc(2, std::iter::empty())?;
        memory.copy_string("41", address1);
        memory.copy_string("42", address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address1);
        vm.stack[1] = Value::String(address2);
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_string_concat() -> Result<(), Error> {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let s1 = String::from("4");
        let s2 = String::from("2");
        let address1 = allocator.malloc(1, std::iter::empty())?;
        let address2 = allocator.malloc(1, std::iter::empty())?;
        memory.copy_string(&s1, address1);
        memory.copy_string(&s2, address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address2);
        vm.stack[1] = Value::String(address1);
        vm.execute_instruction(create_instruction(InstructionType::StringConcat))?;
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
        vm.execute_instruction(create_instruction(InstructionType::Syscall))?;
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
        vm.execute_instruction(create_instruction(InstructionType::SetGlobal(0)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.globals[&0], Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_get_global() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.globals.insert(0, Value::Integer(0));
        vm.execute_instruction(create_instruction(InstructionType::GetGlobal(0)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(0));
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: GlobalDoesntExist(\"4\"), file: \"hola\", line: 0 }"
    )]
    fn test_get_global_not_existing() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let s1 = String::from("4");
        let address1 = allocator.malloc(1, std::iter::empty()).unwrap();
        memory.copy_string(&s1, address1);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.constants = vec![Value::String(address1)];
        vm.execute_instruction(create_instruction(InstructionType::GetGlobal(0)))
            .unwrap();
    }

    #[test]
    fn test_set_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::SetLocal(1)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_get_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::GetLocal(0)))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[1], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_uplift_local() -> Result<(), Error> {
        let memory = Memory::new(110);
        let allocator = Allocator::new(110);
        let mut vm = VM::test_vm_with_memory_and_allocator(1, memory, allocator);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::Uplift(0)))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Pointer(4));
        assert_eq!(vm.stack[1], Value::Pointer(4));
        assert_eq!(*vm.memory.get_t::<Value>(4).unwrap(), Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(0);
        vm.execute_instruction(create_instruction(InstructionType::JmpIfFalse(3)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 4);
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_not_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::JmpIfFalse(3)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 1);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::Jmp(3)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 4);
        Ok(())
    }

    #[test]
    fn test_loop() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.frames[0].ip = 4;
        vm.execute_instruction(create_instruction(InstructionType::Loop(3)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 1);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Function { ip: 20, arity: 1, uplifts: None };
        vm.execute_instruction(create_instruction(InstructionType::Call))?;
        assert_eq!(vm.frames.last().unwrap().stack_offset, 0);
        assert_eq!(vm.frames.len(), 2);
        assert_eq!(vm.ip(), 20);
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: ExpectedNumbers, file: \"hola\", line: 0 }"
    )]
    fn test_call_on_non_function() {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Call))
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: NotEnoughArgumentsForFunction, file: \"hola\", line: 0 }"
    )]
    fn test_call_without_enough_arguments() {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = Value::Function { ip: 20, arity: 2, uplifts: None, };
        vm.execute_instruction(create_instruction(InstructionType::Call))
            .unwrap();
    }

    #[test]
    fn test_array_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::ArrayAlloc))
            .unwrap();
        if let Value::Array { capacity, address } = vm.stack[0] {
            assert_eq!(
                vm.allocator.borrow().get_allocated_space(address).unwrap(),
                capacity * VALUE_SIZE
            );
        } else {
            panic!("Expected array as output of ArrayAlloc {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_array_get() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = Value::Integer(42);
        let address = allocator
            .malloc(std::mem::size_of::<Value>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::Integer(0);
        vm.stack[1] = Value::Array {
            address,
            capacity: 1,
        };
        vm.execute_instruction(create_instruction(InstructionType::ArrayGet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(42));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: IndexOutOfRange, file: \"hola\", line: 0 }"
    )]
    fn test_array_get_out_of_range() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = Value::Integer(42);
        let address = allocator
            .malloc(std::mem::size_of::<Value>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Array {
            address,
            capacity: 1,
        };
        vm.execute_instruction(create_instruction(InstructionType::ArrayGet))
            .unwrap();
    }

    #[test]
    fn test_array_set() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = Value::Integer(42);
        let address = allocator
            .malloc(std::mem::size_of::<Value>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = Value::Integer(0);
        vm.stack[2] = Value::Array {
            address,
            capacity: 1,
        };
        vm.execute_instruction(create_instruction(InstructionType::ArraySet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(
            vm.memory.get_t::<Value>(address).unwrap().clone(),
            Value::Integer(0)
        );
        assert_eq!(vm.stack[0], Value::Integer(0));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: IndexOutOfRange, file: \"hola\", line: 0 }"
    )]
    fn test_array_set_out_of_range() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = Value::Integer(42);
        let address = allocator
            .malloc(std::mem::size_of::<Value>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = Value::Integer(1);
        vm.stack[2] = Value::Array {
            address,
            capacity: 1,
        };
        vm.execute_instruction(create_instruction(InstructionType::ArraySet))
            .unwrap();
    }

    #[test]
    fn test_multi_array_set() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = Value::Integer(42);
        let address = allocator
            .malloc(std::mem::size_of::<Value>() * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        memory.copy_t(&value, address + VALUE_SIZE);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(1);
        vm.stack[1] = Value::Integer(2);
        vm.stack[2] = Value::Array {
            address,
            capacity: 2,
        };
        vm.execute_instruction(create_instruction(InstructionType::MultiArraySet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(
            vm.memory.get_t::<Value>(address).unwrap().clone(),
            Value::Integer(2)
        );
        assert_eq!(
            vm.memory.get_t::<Value>(address + VALUE_SIZE).unwrap().clone(),
            Value::Integer(1)
        );
        assert_eq!(vm.stack[0], Value::Array { address, capacity: 2 });
    }

    #[test]
    fn test_object_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = Value::Integer(1);
        vm.execute_instruction(create_instruction(InstructionType::ObjectAlloc))
            .unwrap();
        if let Value::Object { address } = vm.stack[0] {
            assert_eq!(0usize, *vm.memory.get_t(address).unwrap(),);
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
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(address);
        vm.stack[1] = Value::Object {
            address: obj_address,
        };
        vm.execute_instruction(create_instruction(InstructionType::ObjectGet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(42));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: PropertyDoesntExist(\"VALUE1\"), file: \"hola\", line: 0 }"
    )]
    fn test_object_get_wrong_key() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let wrong_address = allocator.malloc(6, std::iter::empty()).unwrap();
        memory.copy_string("VALUE1", wrong_address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = Value::String(wrong_address);
        vm.stack[1] = Value::Object {
            address: obj_address,
        };
        vm.execute_instruction(create_instruction(InstructionType::ObjectGet))
            .unwrap();
    }

    #[test]
    fn test_object_set() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&0usize, obj_address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address);
        vm.stack[2] = Value::Object {
            address: obj_address,
        };
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm
            .memory
            .get_t::<Value>(obj_address + USIZE_SIZE * 2)
            .unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(address_got, address);
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        assert_eq!(
            vm.stack[1],
            Value::Object {
                address: obj_address
            }
        );
    }

    #[test]
    fn test_object_set_on_existing() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address);
        vm.stack[2] = Value::Object {
            address: obj_address,
        };
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm
            .memory
            .get_t::<Value>(obj_address + USIZE_SIZE * 2)
            .unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(address_got, address);
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        assert_eq!(
            vm.stack[1],
            Value::Object {
                address: obj_address
            }
        );
    }

    #[test]
    fn test_object_set_on_non_existing_without_space() {
        let mut vm = VM::test_vm_with_mem(3, 200);
        let address = vm
            .allocator
            .borrow_mut()
            .malloc(5, std::iter::empty())
            .unwrap();
        vm.memory.copy_string("VALUE", address);
        let address2 = vm
            .allocator
            .borrow_mut()
            .malloc(6, std::iter::empty())
            .unwrap();
        vm.memory.copy_string("VALUE1", address2);
        let obj_address = vm
            .allocator
            .borrow_mut()
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        vm.memory.copy_t(&1usize, obj_address);
        vm.memory.copy_t(&address, obj_address + USIZE_SIZE);
        vm.memory
            .copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        vm.stack[0] = Value::Integer(42);
        vm.stack[1] = Value::String(address2);
        vm.stack[2] = Value::Object {
            address: obj_address,
        };
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], Value::Integer(42));
        if let Value::Object {
            address: obj_address,
        } = &vm.stack[1]
        {
            let obj_address = *obj_address;
            let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
            let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
            let value_got = vm
                .memory
                .get_t::<Value>(obj_address + USIZE_SIZE * 2)
                .unwrap();
            let address_got2 = *vm
                .memory
                .get_t::<usize>(obj_address + USIZE_SIZE * 2 + VALUE_SIZE)
                .unwrap();
            let value_got2 = vm
                .memory
                .get_t::<Value>(obj_address + USIZE_SIZE * 3 + VALUE_SIZE)
                .unwrap();
            assert_eq!(length_got, 2);
            assert_eq!(address_got, address2);
            assert_eq!(address_got2, address);
            assert_eq!(value_got, &Value::Integer(42));
            assert_eq!(value_got2, &Value::Integer(41));
        }
    }

    #[test]
    fn test_attach_uplifts() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.globals.insert(0, Value::Function {
            ip: 0,
            arity: 0,
            uplifts: None
        });
        vm.stack[0] = Value::Array { address: 0, capacity: 0 };
        vm.execute_instruction(create_instruction(InstructionType::AttachArray(0)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.globals.get(&0).cloned(), Some(Value::Function { ip: 0, arity: 0, uplifts: Some(0), }));
        Ok(())
    }
}
