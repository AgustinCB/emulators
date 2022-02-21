use std::borrow::BorrowMut;
use crate::allocator::Allocator;
use crate::instruction::{Instruction, InstructionType};
use crate::memory::Memory;
use failure::Error;
use failure::_core::fmt::Formatter;
use sc::{syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, syscall6};
use std::cell::RefCell;
use std::collections::{HashMap, BTreeSet};
use std::fmt::Display;
use std::iter::FromIterator;

pub(crate) const STACK_MAX: usize = 256;
pub const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const F32_SIZE: usize = std::mem::size_of::<f32>();

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
    Object { address: usize, tags: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompoundValue {
    SimpleValue(Value),
    PartialFunction { function: Value, arguments: Vec<Value>, }
}

fn next_x_items<I: Iterator<Item=u8>>(iterator: &mut I, x: usize) -> Vec<u8> {
    let mut result = vec![];
    for _ in 0..x {
        result.push(iterator.next().unwrap());
    }
    result
}

impl<I: Iterator<Item=u8>> From<&mut I> for Value {
    fn from(bytes: &mut I) -> Self {
        match bytes.next().unwrap() {
            0 => Value::Nil,
            1 => {
                let bytes = next_x_items(bytes, U64_SIZE);
                let integer = *unsafe { (bytes.as_ptr() as *const i64).as_ref() }.unwrap();
                Value::Integer(integer)
            }
            2 => {
                let bytes = next_x_items(bytes, F32_SIZE);
                let float = *unsafe { (bytes.as_ptr() as *const f32).as_ref() }.unwrap();
                Value::Float(float)
            }
            3 => {
                let bool = bytes.next().unwrap() != 0;
                Value::Bool(bool)
            }
            4 => {
                let bytes = next_x_items(bytes, USIZE_SIZE);
                let address = * unsafe { (bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                Value::String(address)
            }
            5 => {
                let ip_bytes = next_x_items(bytes, USIZE_SIZE);
                let ip = * unsafe { (ip_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                let arity_bytes = next_x_items(bytes, USIZE_SIZE);
                let arity = * unsafe { (arity_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                let uplifts = if bytes.next().unwrap() == 0 {
                    None
                } else {
                    let address_bytes = next_x_items(bytes, USIZE_SIZE);
                    Some(
                        * unsafe { (address_bytes.as_ptr() as *const usize).as_ref() }.unwrap()
                    )
                };
                Value::Function { arity, ip, uplifts }
            }
            6 => {
                let capacity_bytes = next_x_items(bytes, USIZE_SIZE);
                let capacity = * unsafe { (capacity_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                let address_bytes = next_x_items(bytes, USIZE_SIZE);
                let address = * unsafe { (address_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                Value::Array { address, capacity }
            }
            7 => {
                let address_bytes = next_x_items(bytes, USIZE_SIZE);
                let address = * unsafe { (address_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                let tags_bytes = next_x_items(bytes, USIZE_SIZE);
                let tags = * unsafe { (tags_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                Value::Object { address, tags }
            }
            8 => {
                let address_bytes = next_x_items(bytes, USIZE_SIZE);
                let address = * unsafe { (address_bytes.as_ptr() as *const usize).as_ref() }.unwrap();
                Value::Pointer(address)
            }
            _ => unimplemented!()
        }
    }
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
            Value::Array { capacity, address } => {
                ret.push(6);
                ret.extend_from_slice(&capacity.to_le_bytes());
                ret.extend_from_slice(&address.to_le_bytes())
            }
            Value::Object { address, tags } => {
                ret.push(7);
                ret.extend_from_slice(&address.to_le_bytes());
                ret.extend_from_slice(&tags.to_le_bytes())
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
const COMPOUND_VALUE_SIZE: usize = std::mem::size_of::<CompoundValue>();
pub(crate) const NULL_VALUE: CompoundValue = CompoundValue::SimpleValue(Value::Nil);
#[cfg(test)]
const ZERO_VALUE: CompoundValue = CompoundValue::SimpleValue(Value::Integer(0));

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

impl Into<bool> for CompoundValue {
    fn into(self) -> bool {
        match self {
            CompoundValue::SimpleValue(sv) => sv.into(),
            CompoundValue::PartialFunction { .. } => true,
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum VMErrorType {
    #[fail(display = "Trying to push to a full stack")]
    StackOverflow,
    #[fail(display = "Trying to pop from an empty stack")]
    EmptyStack,
    #[fail(display = "Expected two numbers. Got {:?} and {:?}", 0, 1)]
    ExpectedNumbers(CompoundValue, CompoundValue),
    #[fail(display = "Expected a number. Got {:?}", 0)]
    ExpectedNumber(CompoundValue),
    #[fail(display = "Expected String")]
    ExpectedString,
    #[fail(display = "Expected two Strings. Got {:?} and {:?}", 0, 1)]
    ExpectedStrings(CompoundValue, CompoundValue),
    #[fail(display = "Expected a function. Got {:?}", 0)]
    ExpectedFunction(CompoundValue),
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
    GlobalDoesntExist(usize),
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
    pub(crate) globals: HashMap<usize, CompoundValue>,
    pub(crate) sp: usize,
    pub(crate) stack: [CompoundValue; STACK_MAX],
    pub debug: bool,
    pub constants: Vec<CompoundValue>,
    pub rom: Vec<Instruction>,
    pub locations: Vec<Location>,
}

impl VM {
    pub fn new(
        allocator: Allocator,
        constants: Vec<CompoundValue>,
        locations: Vec<Location>,
        memory: Memory,
        rom: Vec<Instruction>,
    ) -> VM {
        VM {
            allocator: RefCell::new(allocator),
            frames: vec![],
            globals: HashMap::new(),
            sp: 0,
            stack: [NULL_VALUE; STACK_MAX],
            debug: false,
            constants,
            locations,
            memory,
            rom,
        }
    }

    fn pop(&mut self) -> Result<CompoundValue, Error> {
        if (self.sp - self.frames.last().unwrap().stack_offset) == 0 {
            Err(self.create_error(VMErrorType::EmptyStack)?)?;
        }
        self.sp -= 1;
        Ok(self.stack[self.sp].clone())
    }

    fn peek(&self) -> Result<CompoundValue, Error> {
        if (self.sp - self.frames.last().unwrap().stack_offset) == 0 {
            Err(self.create_error(VMErrorType::EmptyStack)?)?;
        }
        Ok(self.stack[self.sp - 1].clone())
    }

    fn push(&mut self, v: CompoundValue) -> Result<(), Error> {
        if self.sp == self.stack.len() {
            Err(self.create_error(VMErrorType::StackOverflow)?)?;
        }
        self.stack[self.sp] = v;
        self.sp += 1;
        Ok(())
    }

    pub fn stack(&self) -> &[CompoundValue] {
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

    fn dereference_pointer(&self, value: CompoundValue) -> Result<CompoundValue, Error> {
        if let CompoundValue::SimpleValue(Value::Pointer(address)) = value {
            Ok(self.memory.get_t::<CompoundValue>(address)?.clone())
        } else {
            Ok(value)
        }
    }
    fn dereference_pop(&mut self) -> Result<CompoundValue, Error> {
        let value = self.pop()?;
        self.dereference_pointer(value)
    }

    fn switch_context(
        &mut self,
        ip: usize,
        arity: usize,
        uplifts: Option<usize>,
        extra_arguments: Option<&[Value]>,
    ) -> Result<(), Error> {
        let arguments_length = extra_arguments.map_or(0, |args| args.len());
        if (self.sp + arguments_length) < arity {
            Err(self.create_error(VMErrorType::NotEnoughArgumentsForFunction)?)?;
        }
        self.new_frame(ip, arity - arguments_length);
        if let Some(arguments) = extra_arguments {
            for i in (arguments_length..arity).rev() {
                self.get_local(i - arguments_length)?;
                self.set_local(i)?;
            }
            for (i, argument) in arguments.iter().enumerate() {
                self.push(CompoundValue::SimpleValue(argument.clone()))?;
                self.set_local(i)?;
            }
        }
        if let Some(address) = uplifts {
            let array_size = self.get_size(address)? / COMPOUND_VALUE_SIZE;
            let offset = arity;
            for i in 0..array_size {
                let value = self.memory.get_t::<CompoundValue>(address + i * COMPOUND_VALUE_SIZE)?.clone();
                self.push(value)?;
                self.set_local(i + offset)?;
                self.pop()?;
            }
        }
        Ok(())
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
            debug: false,
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
            stack: [ZERO_VALUE; STACK_MAX],
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
            debug: false,
            frames: vec![Frame {
                arity: 0,
                ip: 0,
                stack_offset: 0,
            }],
            globals: HashMap::default(),
            locations: vec![],
            memory: Memory::new(mem),
            stack: [ZERO_VALUE; STACK_MAX],
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
            debug: false,
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
            stack: [ZERO_VALUE; STACK_MAX],
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
    use crate::cpu::CompoundValue;

    #[test]
    fn test_pop() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        let v = vm.pop()?;
        assert_eq!(v, CompoundValue::SimpleValue(Value::Integer(0)));
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
        vm.push(CompoundValue::SimpleValue(Value::Integer(1)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: StackOverflow, file: \"hola\", line: 0 }"
    )]
    fn test_push_on_stack() {
        let mut vm = VM::test_vm(STACK_MAX);
        vm.push(CompoundValue::SimpleValue(Value::Integer(1))).unwrap();
    }
}

macro_rules! comp_operation {
    ($self: ident, $op: tt) => {
        match ($self.dereference_pop()?, $self.dereference_pop()?) {
            (CompoundValue::SimpleValue(Value::Integer(a)), CompoundValue::SimpleValue(Value::Integer(b))) => $self.push(CompoundValue::SimpleValue(Value::Bool(b $op a))),
            (CompoundValue::SimpleValue(Value::Float(a)), CompoundValue::SimpleValue(Value::Integer(b))) => $self.push(CompoundValue::SimpleValue(Value::Bool((b as f32) $op a))),
            (CompoundValue::SimpleValue(Value::Integer(a)), CompoundValue::SimpleValue(Value::Float(b))) => $self.push(CompoundValue::SimpleValue(Value::Bool(b $op (a as f32)))),
            (CompoundValue::SimpleValue(Value::Float(a)), CompoundValue::SimpleValue(Value::Float(b))) => $self.push(CompoundValue::SimpleValue(Value::Bool(b $op a))),
            (CompoundValue::SimpleValue(Value::Bool(a)), CompoundValue::SimpleValue(Value::Bool(b))) => $self.push(CompoundValue::SimpleValue(Value::Bool(b $op a))),
            (CompoundValue::SimpleValue(Value::Bool(a)), v) => {
                let b: bool = v.into();
                $self.push(CompoundValue::SimpleValue(Value::Bool(b $op a)))
            },
            (v, CompoundValue::SimpleValue(Value::Bool(a))) => $self.push(CompoundValue::SimpleValue(Value::Bool(a $op v.into()))),
            (CompoundValue::SimpleValue(Value::String(s1)), CompoundValue::SimpleValue(Value::String(s2))) => {
                let result = {
                    let string1 = $self.memory.get_string(s2, $self.get_size(s2)?)?;
                    let string2 = $self.memory.get_string(s1, $self.get_size(s1)?)?;
                    string1 $op string2
                };
                $self.push(CompoundValue::SimpleValue(Value::Bool(result)))
            },
            (CompoundValue::SimpleValue(Value::Nil), CompoundValue::SimpleValue(Value::Nil)) => $self.push(CompoundValue::SimpleValue(Value::Bool(false))),
            _ => $self.push(CompoundValue::SimpleValue(Value::Bool(false))),
        }?;
    };
}

macro_rules! logical_operation {
    ($self: ident, $op: tt) => {
        let value_a = $self.dereference_pop()?;
        let value_b = $self.dereference_pop()?;
        let a: bool = value_a.into();
        let b: bool = value_b.into();
        $self.push(CompoundValue::SimpleValue(Value::Bool(b $op a)))?;
    };
}

macro_rules! math_operation {
    ($self: ident, $op: tt, $location: expr) => {
        match ($self.dereference_pop()?, $self.dereference_pop()?) {
            (CompoundValue::SimpleValue(Value::Integer(a)), CompoundValue::SimpleValue(Value::Integer(b))) => $self.push(CompoundValue::SimpleValue(Value::Integer(b $op a))),
            (CompoundValue::SimpleValue(Value::Float(a)), CompoundValue::SimpleValue(Value::Integer(b))) => $self.push(CompoundValue::SimpleValue(Value::Float(b as f32 $op a))),
            (CompoundValue::SimpleValue(Value::Integer(a)), CompoundValue::SimpleValue(Value::Float(b))) => $self.push(CompoundValue::SimpleValue(Value::Float(b $op a as f32))),
            (CompoundValue::SimpleValue(Value::Float(a)), CompoundValue::SimpleValue(Value::Float(b))) => $self.push(CompoundValue::SimpleValue(Value::Float(b $op a))),
            (v1, v2) => {
                Err(Error::from($self.create_error(VMErrorType::ExpectedNumbers(v1, v2))?))
            },
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
        if self.debug {
            eprintln!("Instruction: {:?}\tStack: {:?}", instruction, self.stack());
        }
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
            InstructionType::Nil => self.push(CompoundValue::SimpleValue(Value::Nil))?,
            InstructionType::True => self.push(CompoundValue::SimpleValue(Value::Bool(true)))?,
            InstructionType::False => self.push(CompoundValue::SimpleValue(Value::Bool(false)))?,
            InstructionType::Not => {
                let b: bool = self.dereference_pop()?.into();
                self.push(CompoundValue::SimpleValue(Value::Bool(!b)))?;
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
                    CompoundValue::SimpleValue(Value::Integer(a)) => self.push(CompoundValue::SimpleValue(Value::Integer(a.abs())))?,
                    CompoundValue::SimpleValue(Value::Float(a)) => self.push(CompoundValue::SimpleValue(Value::Float(a.abs())))?,
                    v => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
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
            InstructionType::ObjectHas => self.object_has()?,
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
            InstructionType::CheckType(type_index) => self.check_type(*type_index)?,
            InstructionType::AddTag => self.add_tag()?,
            InstructionType::CheckTag => self.check_tag()?,
            InstructionType::ObjectMerge => self.object_merge()?,
            InstructionType::RemoveTag => self.remove_tag()?,
            InstructionType::Duplicate => self.duplicate()?,
        };
        Ok(())
    }

    fn check_type(&mut self, type_index: usize) -> Result<(), Error> {
        let value = self.dereference_pop()?;
        let result = match (value, type_index) {
            (CompoundValue::SimpleValue(Value::Nil), 0) => true,
            (CompoundValue::SimpleValue(Value::Bool(_)), 1) => true,
            (CompoundValue::SimpleValue(Value::Integer(_)), 2) => true,
            (CompoundValue::SimpleValue(Value::Float(_)), 3) => true,
            (CompoundValue::SimpleValue(Value::String(_)), 4) => true,
            (CompoundValue::SimpleValue(Value::Function { .. }), 5) => true,
            (CompoundValue::SimpleValue(Value::Array { .. }), 6) => true,
            (CompoundValue::SimpleValue(Value::Object { .. }), 7) => true,
            _ => false
        };
        self.push(CompoundValue::SimpleValue(Value::Bool(result)))?;
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
            (CompoundValue::SimpleValue(Value::String(s1)), CompoundValue::SimpleValue(Value::String(s2))) => {
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
                self.push(CompoundValue::SimpleValue(Value::String(address)))?;
            }
            (v1, v2) => Err(self.create_error(VMErrorType::ExpectedStrings(v1, v2))?)?,
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
        self.push(CompoundValue::SimpleValue(Value::Integer(ret as _)))?;
        Ok(())
    }

    fn get_global(&mut self, global: usize) -> Result<(), Error> {
        match self.globals.get(&global).cloned() {
            None => {
                Err(self.create_error(VMErrorType::GlobalDoesntExist(global))?)?;
            }
            Some(value) => self.push(value)?,
        };
        Ok(())
    }

    fn set_global(&mut self, global: usize) -> Result<(), Error> {
        let value = self.dereference_pop()?;
        if let Some(CompoundValue::SimpleValue(Value::Pointer(address))) = self.globals.get(&global) {
            let address = *address;
            self.memory.copy_t(&self.peek()?, address);
            self.push(CompoundValue::SimpleValue(Value::Pointer(address)))?;
        } else {
            self.globals.insert(global, value.clone());
            self.push(value)?;
        }
        Ok(())
    }

    fn get_local(&mut self, local: usize) -> Result<(), Error> {
        self.push(self.stack()[self.frames.last().unwrap().stack_offset + local].clone())?;
        Ok(())
    }

    fn set_local(&mut self, local: usize) -> Result<(), Error> {
        let value = self.dereference_pop()?;
        if self.sp - self.frames.last().unwrap().stack_offset == 0 {
            self.sp += local+1;
        }
        if let CompoundValue::SimpleValue(Value::Pointer(address)) = self.stack[self.frames.last().unwrap().stack_offset + local] {
            self.memory.copy_t(&value, address);
            self.push(CompoundValue::SimpleValue(Value::Pointer(address)))?;
        } else {
            self.stack[self.frames.last().unwrap().stack_offset + local] = value.clone();
            if self.frames.last().unwrap().stack_offset + local >= self.sp {
                self.sp += (self.frames.last().unwrap().stack_offset + local + 1) - self.sp;
            }
            self.push(value)?;
        }
        Ok(())
    }

    fn uplift(&mut self, local: usize) -> Result<(), Error> {
        let value = self.stack[self.frames.last().unwrap().stack_offset + local].clone();
        if let CompoundValue::SimpleValue(Value::Pointer(_)) = value {
            self.push(value)?;
        } else {
            let address = self.allocator.borrow_mut().malloc_t::<CompoundValue, _>(self.get_roots())?;
            self.memory.copy_t(&value, address);
            self.stack[self.frames.last().unwrap().stack_offset + local] = CompoundValue::SimpleValue(Value::Pointer(address));
            self.push(CompoundValue::SimpleValue(Value::Pointer(address)))?;
        }
        Ok(())
    }

    fn attach_array(&mut self, global: usize) -> Result<(), Error> {
        let function = self.globals.get(&global).cloned();
        if let None = function {
            return Err(Error::from(self.create_error(VMErrorType::InvalidConstant(global))?));
        }
        if let Some(CompoundValue::SimpleValue(Value::Function { ip, arity, .. })) = function {
            let address = if let CompoundValue::SimpleValue(Value::Array { address, .. }) = self.pop()? {
                address
            } else {
                return Err(Error::from(self.create_error(VMErrorType::ExpectedArray)?));
            };
            self.globals.insert(global, CompoundValue::SimpleValue(Value::Function { ip, arity, uplifts: Some(address) }));
            Ok(())
        } else {
            Err(Error::from(self.create_error(VMErrorType::ExpectedFunction(function.unwrap()))?))
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
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::Function { ip, arity, uplifts }) => {
                self.switch_context(ip, arity, uplifts, None)?;
            },
            CompoundValue::PartialFunction {
                function: Value::Function { ip, arity, uplifts },
                arguments
            } => {
                self.switch_context(ip, arity, uplifts, Some(&arguments))?;
            }
            CompoundValue::SimpleValue(Value::Object { address, tags }) => {
                let address: usize = *self.memory.borrow_mut().get_t(address)?;
                let this = self.create_object(address, tags)?;
                self.push(CompoundValue::SimpleValue(this))?;
            }
            v => Err(self.create_error(VMErrorType::ExpectedFunction(v))?)?,
        };
        Ok(())
    }

    fn array_alloc(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::Integer(capacity)) =>  {
                let address = self
                    .allocator
                    .borrow_mut()
                    .malloc(COMPOUND_VALUE_SIZE * capacity as usize, self.get_roots())?;
                self.push(CompoundValue::SimpleValue(Value::Array {
                    capacity: capacity as usize,
                    address,
                }))?;
            }
            v => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
        }
        Ok(())
    }

    fn array_get(&mut self) -> Result<(), Error> {
        match (self.dereference_pop()?, self.dereference_pop()?) {
            (CompoundValue::SimpleValue(Value::Array { capacity, .. }), CompoundValue::SimpleValue(Value::Integer(index)))
                if capacity <= index as usize =>
            {
                Err(self.create_error(VMErrorType::IndexOutOfRange)?)?
            }
            (CompoundValue::SimpleValue(Value::Array { address, .. }), CompoundValue::SimpleValue(Value::Integer(index))) => {
                let v = self
                    .memory
                    .get_t::<CompoundValue>(address + index as usize * COMPOUND_VALUE_SIZE)?
                    .clone();
                self.push(v)?;
            }
            (CompoundValue::SimpleValue(Value::Array { .. }), v) => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
            (_, _) => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn array_set(&mut self) -> Result<(), Error> {
        match (self.dereference_pop()?, self.dereference_pop()?) {
            (CompoundValue::SimpleValue(Value::Array { capacity, .. }), CompoundValue::SimpleValue(Value::Integer(index)))
                if capacity <= index as usize =>
            {
                Err(self.create_error(VMErrorType::IndexOutOfRange)?)?
            }
            (CompoundValue::SimpleValue(Value::Array { address, .. }), CompoundValue::SimpleValue(Value::Integer(index))) => {
                let v = self.peek()?;
                self.memory
                    .copy_t::<CompoundValue>(&v, address + index as usize * COMPOUND_VALUE_SIZE);
            }
            (CompoundValue::SimpleValue(Value::Array { .. }), v) => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
            (_, _) => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn multi_array_set(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::Array { address, capacity }) => {
                let mut vs = vec![];
                for _ in 0..capacity {
                    let v = self.pop()?;
                    vs.push(v);
                }
                self.memory.copy_t_slice(&vs, address);
                self.push(CompoundValue::SimpleValue(Value::Array { address, capacity }))?;
            }
            _ => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn repeated_array_set(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::Array { address, capacity }) => {
                let v = self.pop()?;
                let vs = vec![v].into_iter().cycle().take(capacity).collect::<Vec<CompoundValue>>();
                self.memory.copy_t_slice(&vs, address);
                self.push(CompoundValue::SimpleValue(Value::Array { address, capacity }))?;
            }
            _ => Err(self.create_error(VMErrorType::ExpectedArray)?)?,
        };
        Ok(())
    }

    fn object_alloc(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::Integer(capacity)) => {
                let capacity = (VALUE_SIZE + USIZE_SIZE) * capacity as usize;
                let size = capacity + USIZE_SIZE;
                let address = self.allocator.borrow_mut().malloc(USIZE_SIZE, self.get_roots())?;
                let props_address = self.allocator.borrow_mut().malloc(size, self.get_roots())?;
                let tags = self.allocator.borrow_mut().malloc(USIZE_SIZE, self.get_roots())?;
                self.memory.copy_t(&0usize, tags);
                self.memory.copy_t(&0usize, props_address);
                self.memory.copy_t(&props_address, address);
                self.push(CompoundValue::SimpleValue(Value::Object { address, tags }))?;
            }
            v => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
        }
        Ok(())
    }

    fn object_get(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(this_value@Value::Object {
                address: obj_address,
                ..
            }),
            CompoundValue::SimpleValue(Value::String(address)),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let size = self
                .allocator
                .borrow()
                .get_allocated_space(address)
                .unwrap();
            let property = self.memory.get_string(address, size)?;
            let bytes = self.get_properties(obj_address)?;
            let i = match self.property_lookup(bytes, property) {
                Ok(i) => i,
                Err(_) => {
                    Err(self.create_error(VMErrorType::PropertyDoesntExist(property.to_owned()))?)?
                }
            };
            let value = bytes[i].1;
            if let Value::Function { .. } = bytes[i].1 {
                self.push(CompoundValue::PartialFunction {
                    function: value,
                    arguments: vec![this_value]
                })?;
            } else {
                self.push(CompoundValue::SimpleValue(value))?;
            }
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?;
        }
        Ok(())
    }

    fn object_set(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(Value::Object {
                address: obj_prop_address,
                tags,
            }),
            CompoundValue::SimpleValue(Value::String(address)),
            CompoundValue::SimpleValue(value),
        ) = (self.dereference_pop()?, self.dereference_pop()?, self.pop()?)
        {
            let mut obj_address: usize = *self.memory.borrow_mut().get_t(obj_prop_address)?;
            let capacity = (self.get_size(obj_address)? - USIZE_SIZE) / (VALUE_SIZE + USIZE_SIZE);
            let size = self.get_size(address)?;
            let property = self.memory.get_string(address, size)?;
            let bytes = self.get_properties(obj_prop_address)?;
            let index = match self.property_lookup(bytes, property) {
                Ok(index) => index,
                Err(index) => {
                    let object_length: usize = *self.memory.get_t(obj_address)?;
                    if capacity <= object_length {
                        self.allocator.borrow_mut().free(obj_address)?;
                        obj_address = self.allocator.borrow_mut().malloc(
                            USIZE_SIZE + capacity * 2 * (VALUE_SIZE + USIZE_SIZE),
                            self.get_roots(),
                        )?;
                        self.memory.copy_t(&obj_address, obj_prop_address);
                        self.memory.copy_t(&(object_length + 1), obj_address);
                        self.memory.copy_t_slice(&bytes, obj_address + USIZE_SIZE);
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
            self.push(CompoundValue::SimpleValue(value))?;
            self.push(CompoundValue::SimpleValue(Value::Object {
                address: obj_prop_address,
                tags,
            }))?;
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?;
        }
        Ok(())
    }

    fn object_has(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(this@Value::Object {
                address: obj_address,
                ..
            }),
            CompoundValue::SimpleValue(Value::String(address)),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let size = self
                .allocator
                .borrow()
                .get_allocated_space(address)
                .unwrap();
            let property = self.memory.get_string(address, size)?;
            let bytes = self.get_properties(obj_address)?;
            let has_prop = self.property_lookup(bytes, property).is_ok();
            self.push(CompoundValue::SimpleValue(this))?;
            self.push(CompoundValue::SimpleValue(Value::Bool(has_prop)))?;
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?;
        }
        Ok(())
    }

    fn strlen(&mut self) -> Result<(), Error> {
        match self.dereference_pop()? {
            CompoundValue::SimpleValue(Value::String(s)) => {
                let s_size = self.get_size(s)?;
                self.push(CompoundValue::SimpleValue(Value::Integer(s_size as _)))?;
            },
            _ => Err(self.create_error(VMErrorType::ExpectedString)?)?,
        };
        Ok(())
    }

    fn duplicate(&mut self) -> Result<(), Error> {
        let last = self.peek()?;
        self.push(last)?;
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
        if let CompoundValue::SimpleValue(Value::String(address)) = v {
            self.push(CompoundValue::SimpleValue(Value::String(address)))?;
        } else {
            let s = match v {
                CompoundValue::SimpleValue(Value::Nil) => "nil".to_string(),
                CompoundValue::SimpleValue(Value::Integer(i)) => i.to_string(),
                CompoundValue::SimpleValue(Value::Bool(b)) => b.to_string(),
                CompoundValue::SimpleValue(Value::Float(f)) => f.to_string(),
                CompoundValue::SimpleValue(Value::Function { .. }) => "[function]".to_string(),
                CompoundValue::SimpleValue(Value::Array { .. }) => "[array]".to_string(),
                CompoundValue::SimpleValue(Value::Object { address, .. }) => format!("[object {}]", address),
                CompoundValue::PartialFunction { .. } => "[partial function]".to_string(),
                v => panic!("Cannot convert {:?} to string", v),
            };
            let a = self.allocator.borrow_mut().malloc(s.len(), self.get_roots())?;
            self.memory.copy_u8_vector(s.as_bytes(), a);
            self.push(CompoundValue::SimpleValue(Value::String(a)))?;
        }
        Ok(())
    }

    fn add_tag(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(o@Value::Object { tags, address }),
            CompoundValue::SimpleValue(Value::String(string_address)),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let tags = self.get_tags(tags)?;
            match tags.binary_search(&string_address) {
                Ok(_) => {
                    self.push(CompoundValue::SimpleValue(o))?;
                },
                Err(index) => {
                    let mut new_tags = tags[..index].to_vec();
                    new_tags.push(string_address);
                    new_tags.extend_from_slice(&tags[index..]);
                    let new_tags_address = self.allocator
                        .borrow_mut()
                        .malloc(USIZE_SIZE * new_tags.len(), self.get_roots())?;
                    self.memory.copy_t_slice(&new_tags, new_tags_address);
                    self.push(CompoundValue::SimpleValue(
                        Value::Object { tags: new_tags_address, address }
                    ))?;
                }
            }
            Ok(())
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?
        }
    }

    fn check_tag(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(Value::Object { tags, .. }),
            CompoundValue::SimpleValue(Value::String(string_address)),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            match self.get_tags(tags)?.binary_search(&string_address) {
                Ok(_) => {
                    self.push(CompoundValue::SimpleValue(Value::Bool(true)))?;
                },
                Err(_) => {
                    self.push(CompoundValue::SimpleValue(Value::Bool(false)))?;
                }
            }
            Ok(())
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?
        }
    }

    fn remove_tag(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(o@Value::Object { tags, address }),
            CompoundValue::SimpleValue(Value::String(string_address)),
        ) = (self.dereference_pop()?, self.dereference_pop()?)
        {
            let tags = self.get_tags(tags)?;
            match tags.binary_search(&string_address) {
                Ok(i) => {
                    let length = tags.len() - 1;
                    let new_tags = self.allocator.borrow_mut().malloc(length * USIZE_SIZE, self.get_roots())?;
                    self.memory.copy_t_slice(&tags[0..i], new_tags);
                    self.memory.copy_t_slice(&tags[i+1..], new_tags + i * USIZE_SIZE);
                    self.push(CompoundValue::SimpleValue(Value::Object {
                        address,
                        tags: new_tags
                    }))?;
                },
                Err(_) => {
                    self.push(CompoundValue::SimpleValue(o))?;
                }
            }
            Ok(())
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?
        }
    }

    fn object_merge(&mut self) -> Result<(), Error> {
        if let (
            CompoundValue::SimpleValue(Value::Object { address: second_address, tags: second_tags, .. }),
            CompoundValue::SimpleValue(Value::Object { address: first_address, tags: first_tags, .. }),
        ) = (self.dereference_pop()?, self.dereference_pop()?) {
            let second_properties = self.get_properties(second_address)?;
            let first_properties = self.get_properties(first_address)?;
            let properties = self.merge_properties(first_properties, second_properties)?;
            let new_tags = self.merge_tags(first_tags, second_tags)?;
            let capacity = properties.len() * (VALUE_SIZE + USIZE_SIZE);
            let props_address = self.allocator.borrow_mut().malloc(USIZE_SIZE + capacity, self.get_roots())?;
            let address = self.allocator.borrow_mut().malloc(USIZE_SIZE, self.get_roots())?;
            let tags_capacity = new_tags.len() * USIZE_SIZE;
            let tags = self.allocator.borrow_mut().malloc(tags_capacity, self.get_roots())?;
            self.memory.copy_t(&props_address, address);
            self.memory.copy_t(&properties.len(), props_address);
            self.memory.copy_t_slice(&properties, props_address + USIZE_SIZE);
            self.memory.copy_t_slice(&new_tags, tags);
            self.push(CompoundValue::SimpleValue(Value::Object {
                address,
                tags,
            }))?;
            Ok(())
        } else {
            Err(self.create_error(VMErrorType::ExpectedString)?)?
        }
    }

    fn get_properties(&self, obj_address: usize) -> Result<&[(usize, Value)], Error> {
        let props_address: usize = *self.memory.get_t(obj_address)?;
        let object_length: usize = *self.memory.get_t(props_address)?;
        Ok(self.memory.get_vector::<(usize, Value)>(
            props_address + USIZE_SIZE,
            object_length * (VALUE_SIZE + USIZE_SIZE),
        )?)
    }

    fn get_tags(&self, tags: usize) -> Result<&[usize], Error> {
        let length = self.get_size(tags)?;
        Ok(self.memory.get_vector::<usize>(tags, length)?)
    }

    fn property_lookup(&self, bytes: &[(usize, Value)], property: &str) -> Result<usize, usize> {
        bytes.binary_search_by(|(curr_address, _)| {
            let found_property = self.address_to_string(*curr_address).unwrap();
            found_property.cmp(property)
        })
    }

    fn create_object(&mut self, address: usize, tags: usize) -> Result<Value, Error> {
        let size = self.get_size(address)?;
        let new_props_address = self.allocator.borrow_mut().malloc(size, self.get_roots())?;
        let object_bytes = self.memory.get_u8_vector(address, size)?;
        self.memory.copy_u8_vector(object_bytes, new_props_address);
        let new_address = self.allocator.borrow_mut().malloc(USIZE_SIZE, self.get_roots())?;
        self.memory.copy_t(&new_props_address, new_address);
        let this = Value::Object {
            address: new_address,
            tags,
        };
        Ok(this)
    }

    fn merge_tags(&self, first_tags: usize, second_tags: usize) -> Result<Vec<usize>, Error> {
        let first_tags = self.get_tags(first_tags)?;
        let second_tags = self.get_tags(second_tags)?;
        let first_tags_tree = BTreeSet::from_iter(first_tags.iter().cloned());
        let second_tags_tree = BTreeSet::from_iter(second_tags.iter().cloned());
        Ok(
            first_tags_tree.union(&second_tags_tree)
                .cloned()
                .collect()
        )
    }

    fn merge_properties(
        &self,
        first_properties: &[(usize, Value)],
        second_properties: &[(usize, Value)],
    ) -> Result<Vec<(usize, Value)>, Error> {
        let mut merged_properties = vec![];
        if first_properties.is_empty() && second_properties.is_empty() {
            return Ok(merged_properties);
        }
        if first_properties.is_empty() {
            return Ok(second_properties.to_vec());
        }
        if second_properties.is_empty() {
            return Ok(first_properties.to_vec());
        }
        let mut first_properties_vec = first_properties.to_vec();
        first_properties_vec.reverse();
        let mut second_properties_vec = second_properties.to_vec();
        second_properties_vec.reverse();
        while !first_properties_vec.is_empty() || !second_properties_vec.is_empty() {
            if first_properties_vec.is_empty() {
                second_properties_vec.reverse();
                merged_properties.extend_from_slice(&second_properties_vec);
                break;
            }
            if second_properties_vec.is_empty() {
                first_properties_vec.reverse();
                merged_properties.extend_from_slice(&first_properties_vec);
                break;
            }
            let (first_address, first_value) = first_properties_vec.pop().unwrap();
            let (second_address, second_value) = second_properties_vec.pop().unwrap();
            let first_property = self.address_to_string(first_address)?;
            let second_property = self.address_to_string(second_address)?;
            if first_property < second_property {
                merged_properties.push((first_address, first_value));
                second_properties_vec.push((second_address, second_value));
            } else if second_property < first_property {
                merged_properties.push((second_address, second_value));
                first_properties_vec.push((first_address, first_value));
            } else {
                merged_properties.push((first_address, first_value));
            }
        }
        Ok(merged_properties)
    }

    fn address_to_string(&self, address: usize) -> Result<&str, Error> {
        let found_length = self.get_size(address)?;
        Ok(self.memory.get_string(address, found_length)?)
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
            CompoundValue::SimpleValue(Value::Integer(a)) => a as usize,
            CompoundValue::SimpleValue(Value::Float(f)) => f as usize,
            CompoundValue::SimpleValue(Value::String(address)) => {
                let size = self.get_size(address)?;
                let bs = self.memory.get_u8_vector(address, size)?;
                bs.as_ptr() as usize
            }
            v => Err(self.create_error(VMErrorType::ExpectedNumber(v))?)?,
        };
        Ok(ret)
    }

    fn get_roots<'a>(&'a self) -> impl Iterator<Item = usize> + 'a {
        self.stack
            .iter()
            .chain(self.constants.iter())
            .chain(self.globals.values())
            .filter_map(move |v| match v {
                CompoundValue::SimpleValue(Value::String(address)) => Some(vec![*address]),
                CompoundValue::SimpleValue(Value::Array { address, capacity }) => {
                    Some(self.get_addresses_from_array(*address, *capacity))
                }
                CompoundValue::SimpleValue(Value::Object {address, tags }) =>
                    Some(self.get_addresses_from_object(*address, *tags)),
                _ => None,
            })
            .flatten()
    }

    fn get_addresses_from_object(&self, address: usize, tags: usize) -> Vec<usize> {
        let props_address: usize = *self.memory.get_t(address).unwrap();
        let length: usize = *self.memory.get_t(props_address).unwrap();
        let mut result = vec![address, props_address, tags];
        let pairs = self.memory
            .get_vector::<(usize, Value)>(props_address + USIZE_SIZE,length * (VALUE_SIZE + USIZE_SIZE))
            .unwrap();
        for (string, value) in pairs {
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
            Value::Object { address, tags } => result.extend(self.get_addresses_from_object(*address, *tags)),
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
    use crate::cpu::{USIZE_SIZE, VALUE_SIZE, CompoundValue, COMPOUND_VALUE_SIZE};
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
        vm.constants.push(CompoundValue::SimpleValue(Value::Integer(1)));
        vm.execute_instruction(create_instruction(InstructionType::Constant(0)))?;
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(3)));
        Ok(())
    }

    #[test]
    fn test_add_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(2.0));
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(3.0)));
        Ok(())
    }

    #[test]
    fn test_add_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(3.0)));
        Ok(())
    }

    #[test]
    fn test_add_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Plus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(3.0)));
        Ok(())
    }

    #[test]
    fn test_sub_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    fn test_sub_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(2.0));
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(1.0)));
        Ok(())
    }

    #[test]
    fn test_sub_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(1.0)));
        Ok(())
    }

    #[test]
    fn test_sub_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Minus))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(1.0)));
        Ok(())
    }

    #[test]
    fn test_mult_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(2)));
        Ok(())
    }

    #[test]
    fn test_mult_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(2.0));
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_mult_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_mult_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Mult))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_div_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(2)));
        Ok(())
    }

    #[test]
    fn test_div_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Float(2.0));
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_div_float_integer() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_div_integer_float() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Float(1.0));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.execute_instruction(create_instruction(InstructionType::Div))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Float(2.0)));
        Ok(())
    }

    #[test]
    fn test_nil() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::Nil))?;
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Nil));
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::True))?;
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.execute_instruction(create_instruction(InstructionType::False))?;
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.execute_instruction(create_instruction(InstructionType::Not))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        vm.execute_instruction(create_instruction(InstructionType::Not))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_not_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::NotEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_not_equals_diff() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::NotEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_greater_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_greater_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_greater_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(-1));
        vm.execute_instruction(create_instruction(InstructionType::Greater))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_greater_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_greater_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_greater_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(-1));
        vm.execute_instruction(create_instruction(InstructionType::GreaterEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_less_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_less_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_less_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(-1));
        vm.execute_instruction(create_instruction(InstructionType::Less))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_less_equals_same() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_less_equals_greater() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_less_equals_lesser() -> Result<(), Error> {
        let mut vm = VM::test_vm(2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(-1));
        vm.execute_instruction(create_instruction(InstructionType::LessEqual))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
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
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(address1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(address2));
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
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
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(address1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(address2));
        vm.execute_instruction(create_instruction(InstructionType::Equal))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
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
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(address2));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(address1));
        vm.execute_instruction(create_instruction(InstructionType::StringConcat))?;
        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::String(address)) = vm.stack[0] {
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
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(sc::nr::GETPID as _));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(0));
        vm.execute_instruction(create_instruction(InstructionType::Syscall))?;
        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Integer(n)) = vm.stack[0] {
            assert!(n > 0);
        } else {
            panic!("Syscall should return an integer");
        }
        Ok(())
    }

    #[test]
    fn test_set_global() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(0));
        vm.execute_instruction(create_instruction(InstructionType::SetGlobal(0)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.globals[&0], CompoundValue::SimpleValue(Value::Integer(0)));
        Ok(())
    }

    #[test]
    fn test_get_global() -> Result<(), Error> {
        let mut vm = VM::test_vm(0);
        vm.globals.insert(0, CompoundValue::SimpleValue(Value::Integer(0)));
        vm.execute_instruction(create_instruction(InstructionType::GetGlobal(0)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(0)));
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: GlobalDoesntExist(0), file: \"hola\", line: 0 }"
    )]
    fn test_get_global_not_existing() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let s1 = String::from("4");
        let address1 = allocator.malloc(1, std::iter::empty()).unwrap();
        memory.copy_string(&s1, address1);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.constants = vec![CompoundValue::SimpleValue(Value::String(address1))];
        vm.execute_instruction(create_instruction(InstructionType::GetGlobal(0)))
            .unwrap();
    }

    #[test]
    fn test_set_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::SetLocal(0)))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(1)));
        assert_eq!(vm.stack[1], CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    fn test_get_local() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::GetLocal(0)))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[1], CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    fn test_uplift_local() -> Result<(), Error> {
        let memory = Memory::new(110);
        let allocator = Allocator::new(110);
        let mut vm = VM::test_vm_with_memory_and_allocator(1, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::Uplift(0)))?;
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Pointer(4)));
        assert_eq!(vm.stack[1], CompoundValue::SimpleValue(Value::Pointer(4)));
        assert_eq!(*vm.memory.get_t::<CompoundValue>(4).unwrap(), CompoundValue::SimpleValue(Value::Integer(1)));
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(0));
        vm.execute_instruction(create_instruction(InstructionType::JmpIfFalse(3)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.ip(), 4);
        Ok(())
    }

    #[test]
    fn test_jmp_if_false_not_jmping() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
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
        vm.stack[1] = CompoundValue::SimpleValue(Value::Function { ip: 20, arity: 1, uplifts: None });
        vm.execute_instruction(create_instruction(InstructionType::Call))?;
        assert_eq!(vm.frames.last().unwrap().stack_offset, 0);
        assert_eq!(vm.frames.len(), 2);
        assert_eq!(vm.ip(), 20);
        Ok(())
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: ExpectedFunction(SimpleValue(Integer(0))), file: \"hola\", line: 0 }"
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
        vm.stack[1] = CompoundValue::SimpleValue(Value::Function { ip: 20, arity: 2, uplifts: None, });
        vm.execute_instruction(create_instruction(InstructionType::Call))
            .unwrap();
    }

    #[test]
    fn test_array_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::ArrayAlloc))
            .unwrap();
        if let CompoundValue::SimpleValue(Value::Array { capacity, address }) = vm.stack[0] {
            assert_eq!(
                vm.allocator.borrow().get_allocated_space(address).unwrap(),
                capacity * COMPOUND_VALUE_SIZE
            );
        } else {
            panic!("Expected array as output of ArrayAlloc {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_array_get() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = CompoundValue::SimpleValue(Value::Integer(42));
        let address = allocator
            .malloc(std::mem::size_of::<CompoundValue>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(0));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Array {
            address,
            capacity: 1,
        });
        vm.execute_instruction(create_instruction(InstructionType::ArrayGet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(42)));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: IndexOutOfRange, file: \"hola\", line: 0 }"
    )]
    fn test_array_get_out_of_range() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = CompoundValue::SimpleValue(Value::Integer(42));
        let address = allocator
            .malloc(std::mem::size_of::<CompoundValue>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Array {
            address,
            capacity: 1,
        });
        vm.execute_instruction(create_instruction(InstructionType::ArrayGet))
            .unwrap();
    }

    #[test]
    fn test_array_set() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = CompoundValue::SimpleValue(Value::Integer(42));
        let address = allocator
            .malloc(std::mem::size_of::<CompoundValue>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(0));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Array {
            address,
            capacity: 1,
        });
        vm.execute_instruction(create_instruction(InstructionType::ArraySet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(
            vm.memory.get_t::<CompoundValue>(address).unwrap().clone(),
            CompoundValue::SimpleValue(Value::Integer(0))
        );
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(0)));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: IndexOutOfRange, file: \"hola\", line: 0 }"
    )]
    fn test_array_set_out_of_range() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let value = CompoundValue::SimpleValue(Value::Integer(42));
        let address = allocator
            .malloc(std::mem::size_of::<CompoundValue>(), std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Array {
            address,
            capacity: 1,
        });
        vm.execute_instruction(create_instruction(InstructionType::ArraySet))
            .unwrap();
    }

    #[test]
    fn test_multi_array_set() {
        let memory = Memory::new(150);
        let mut allocator = Allocator::new(150);
        let value = CompoundValue::SimpleValue(Value::Integer(42));
        let address = allocator
            .malloc(std::mem::size_of::<CompoundValue>() * 2, std::iter::empty())
            .unwrap();
        memory.copy_t(&value, address);
        memory.copy_t(&value, address + VALUE_SIZE);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Integer(2));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Array {
            address,
            capacity: 2,
        });
        vm.execute_instruction(create_instruction(InstructionType::MultiArraySet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(
            vm.memory.get_t::<CompoundValue>(address).unwrap().clone(),
            CompoundValue::SimpleValue(Value::Integer(2))
        );
        assert_eq!(
            vm.memory.get_t::<CompoundValue>(address + COMPOUND_VALUE_SIZE).unwrap().clone(),
            CompoundValue::SimpleValue(Value::Integer(1))
        );
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Array { address, capacity: 2 }));
    }

    #[test]
    fn test_object_alloc() {
        let mut vm = VM::test_vm_with_mem(1, 100);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(1));
        vm.execute_instruction(create_instruction(InstructionType::ObjectAlloc))
            .unwrap();
        if let CompoundValue::SimpleValue(Value::Object { address, tags }) = vm.stack[0] {
            let address: usize = *vm.memory.get_t(address).unwrap();
            assert_eq!(0usize, *vm.memory.get_t::<usize>(address).unwrap(),);
            assert_eq!(
                vm.allocator.borrow().get_allocated_space(address).unwrap(),
                VALUE_SIZE + USIZE_SIZE * 2,
            );
            assert_eq!(VALUE_SIZE + USIZE_SIZE * 3, tags);
        } else {
            panic!("Expected array as output of ArrayAlloc {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_object_get() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let string_address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", string_address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        let address = allocator
            .malloc(USIZE_SIZE, std::iter::empty())
            .unwrap();
        memory.copy_t(&obj_address, address);
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&string_address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(string_address));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: 0,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectGet))
            .unwrap();
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(42)));
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: VMError { error_type: PropertyDoesntExist(\"VALUE1\"), file: \"hola\", line: 0 }"
    )]
    fn test_object_get_wrong_key() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let string_address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", string_address);
        let wrong_address = allocator.malloc(6, std::iter::empty()).unwrap();
        memory.copy_string("VALUE1", wrong_address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        let address = allocator
            .malloc(USIZE_SIZE, std::iter::empty())
            .unwrap();
        memory.copy_t(&obj_address, address);
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&string_address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(42), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(wrong_address));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: 0,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectGet))
            .unwrap();
    }

    #[test]
    fn test_object_set() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let string_address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", string_address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        let address = allocator
            .malloc(USIZE_SIZE, std::iter::empty())
            .unwrap();
        memory.copy_t(&obj_address, address);
        memory.copy_t(&0usize, obj_address);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(42));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(string_address));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: 0,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(42)));
        assert_eq!(
            vm.stack[1],
            CompoundValue::SimpleValue(Value::Object {
                address,
                tags: 0,
            })
        );
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm
            .memory
            .get_t::<Value>(obj_address + USIZE_SIZE * 2)
            .unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(address_got, string_address);
    }

    #[test]
    fn test_object_set_on_existing() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let string_address = allocator.malloc(5, std::iter::empty()).unwrap();
        memory.copy_string("VALUE", string_address);
        let obj_address = allocator
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        let address = allocator
            .malloc(USIZE_SIZE, std::iter::empty())
            .unwrap();
        memory.copy_t(&obj_address, address);
        memory.copy_t(&1usize, obj_address);
        memory.copy_t(&string_address, obj_address + USIZE_SIZE);
        memory.copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        let mut vm = VM::test_vm_with_memory_and_allocator(3, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(42));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(string_address));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: 0,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        let length_got = *vm.memory.get_t::<usize>(obj_address).unwrap();
        let address_got = *vm.memory.get_t::<usize>(obj_address + USIZE_SIZE).unwrap();
        let value_got = vm
            .memory
            .get_t::<Value>(obj_address + USIZE_SIZE * 2)
            .unwrap();
        assert_eq!(length_got, 1);
        assert_eq!(address_got, string_address);
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(42)));
        assert_eq!(value_got, &Value::Integer(42));
        assert_eq!(
            vm.stack[1],
            CompoundValue::SimpleValue(Value::Object {
                address,
                tags: 0,
            })
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
        let obj_address = vm.allocator
            .borrow_mut()
            .malloc(VALUE_SIZE + USIZE_SIZE * 2, std::iter::empty())
            .unwrap();
        let address = vm.allocator
            .borrow_mut()
            .malloc(USIZE_SIZE, std::iter::empty())
            .unwrap();
        vm.memory.copy_t(&obj_address, address);
        vm.memory.copy_t(&1usize, obj_address);
        vm.memory.copy_t(&address, obj_address + USIZE_SIZE);
        vm.memory
            .copy_t(&Value::Integer(41), obj_address + USIZE_SIZE * 2);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Integer(42));
        vm.stack[1] = CompoundValue::SimpleValue(Value::String(address2));
        vm.stack[2] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: 0,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectSet))
            .unwrap();
        assert_eq!(vm.sp, 2);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Integer(42)));
        if let CompoundValue::SimpleValue(Value::Object {
            address: obj_address,
            tags: 0,
        }) = &vm.stack[1]
        {
            let obj_address = *obj_address;
            let obj_address = *vm.memory.get_t::<usize>(obj_address).unwrap();
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
            assert_eq!(address_got, address);
            assert_eq!(address_got2, address2);
            assert_eq!(value_got, &Value::Integer(41));
            assert_eq!(value_got2, &Value::Integer(42));
        }
    }

    #[test]
    fn test_attach_uplifts() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.globals.insert(0, CompoundValue::SimpleValue(Value::Function {
            ip: 0,
            arity: 0,
            uplifts: None
        }));
        vm.stack[0] = CompoundValue::SimpleValue(Value::Array { address: 0, capacity: 0 });
        vm.execute_instruction(create_instruction(InstructionType::AttachArray(0)))?;
        assert_eq!(vm.sp, 0);
        assert_eq!(vm.globals.get(&0).cloned(), Some(CompoundValue::SimpleValue(Value::Function { ip: 0, arity: 0, uplifts: Some(0), })));
        Ok(())
    }

    #[test]
    fn test_check_type() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Nil);
        vm.execute_instruction(create_instruction(InstructionType::CheckType(0)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(true)));
        Ok(())
    }

    #[test]
    fn test_check_type_failing() -> Result<(), Error> {
        let mut vm = VM::test_vm(1);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Nil);
        vm.execute_instruction(create_instruction(InstructionType::CheckType(1)))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], CompoundValue::SimpleValue(Value::Bool(false)));
        Ok(())
    }

    #[test]
    fn test_add_tags_on_empty_array() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(0, std::iter::empty()).unwrap();
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(142));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::AddTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
            tags, address: 0
        }) = vm.stack[0] {
            let string_address = *vm.memory.get_t::<usize>(tags).unwrap();
            assert_eq!(string_address, 142);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_add_tags_on_non_empty_array() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t_slice(&[142usize, 144], address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(143));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::AddTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
            tags, address: 0
        }) = vm.stack[0] {
            assert_eq!(
                Some(3 * USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            let string_address = *vm.memory.get_t::<usize>(tags).unwrap();
            assert_eq!(string_address, 142);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE).unwrap();
            assert_eq!(string_address, 143);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE * 2).unwrap();
            assert_eq!(string_address, 144);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_add_tags_on_array_with_duplicated() {
        let memory = Memory::new(110);
        let mut allocator = Allocator::new(110);
        let address = allocator.malloc(USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t_slice(&[142usize, 143], address);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(142));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::AddTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
            tags, address: 0
        }) = vm.stack[0] {
            assert_eq!(
                Some(2 * USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            let string_address = *vm.memory.get_t::<usize>(tags).unwrap();
            assert_eq!(string_address, 142);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE).unwrap();
            assert_eq!(string_address, 143);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_remove_tags_on_empty_array() {
        let memory = Memory::new(110);
        let allocator = Allocator::new(110);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        let address = vm.allocator.borrow_mut().malloc(0, std::iter::empty()).unwrap();
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(142));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::RemoveTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
                                              tags, address: 0
                                          }) = vm.stack[0] {
            let size = vm.allocator.borrow_mut().get_allocated_space(tags).unwrap();
            assert_eq!(size, 0);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_remove_tags_on_non_empty_array() {
        let memory = Memory::new(110);
        let allocator = Allocator::new(110);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        let address = vm.allocator.borrow_mut().malloc(USIZE_SIZE * 3, std::iter::empty()).unwrap();
        vm.memory.copy_t_slice(&[142usize, 143, 144], address);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(143));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::RemoveTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
                                              tags, address: 0
                                          }) = vm.stack[0] {
            assert_eq!(
                Some(2 * USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            let string_address = *vm.memory.get_t::<usize>(tags).unwrap();
            assert_eq!(string_address, 142);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE * 1).unwrap();
            assert_eq!(string_address, 144);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_remove_tag_with_tag_not_there() {
        let memory = Memory::new(110);
        let allocator = Allocator::new(110);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        let address = vm.allocator.borrow_mut().malloc(USIZE_SIZE * 2, std::iter::empty()).unwrap();
        vm.memory.copy_t_slice(&[142usize, 144], address);
        vm.stack[0] = CompoundValue::SimpleValue(Value::String(143));
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: 0,
            tags: address,
        });
        vm.execute_instruction(create_instruction(InstructionType::RemoveTag))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
                                              tags, address: 0
                                          }) = vm.stack[0] {
            assert_eq!(
                Some(2 * USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            assert_eq!(tags, address);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_object_merge_merges_tags() {
        let memory = Memory::new(220);
        let mut allocator = Allocator::new(220);
        let address = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let obj_address = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let tags_address = allocator.malloc(USIZE_SIZE * 2, std::iter::empty()).unwrap();
        let address2 = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let obj_address2 = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let tags_address2 = allocator.malloc(USIZE_SIZE * 2, std::iter::empty()).unwrap();
        memory.copy_t(&obj_address, address);
        memory.copy_t(&0usize, obj_address);
        memory.copy_t_slice(&[142usize, 144], tags_address);
        memory.copy_t(&obj_address2, address2);
        memory.copy_t(&0usize, obj_address2);
        memory.copy_t_slice(&[143usize, 144], tags_address2);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: tags_address,
        });
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: address2,
            tags: tags_address2,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectMerge))
            .unwrap();
        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
                                              tags, address
                                          }) = vm.stack[0] {
            let address = *vm.memory.get_t::<usize>(address).unwrap();
            assert_eq!(
                Some(3 * USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            assert_eq!(
                Some(USIZE_SIZE),
                vm.allocator.borrow().get_allocated_space(address)
            );
            let object_length = *vm.memory.get_t::<usize>(address).unwrap();
            assert_eq!(object_length, 0);
            let string_address = *vm.memory.get_t::<usize>(tags).unwrap();
            assert_eq!(string_address, 142);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE).unwrap();
            assert_eq!(string_address, 143);
            let string_address = *vm.memory.get_t::<usize>(tags + USIZE_SIZE * 2).unwrap();
            assert_eq!(string_address, 144);
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }

    #[test]
    fn test_object_merge_merges_properties() {
        let memory = Memory::new(440);
        let mut allocator = Allocator::new(440);
        let prop_address = allocator.malloc(1, std::iter::empty()).unwrap();
        let prop1_address = allocator.malloc(1, std::iter::empty()).unwrap();
        let prop2_address = allocator.malloc(1, std::iter::empty()).unwrap();
        let prop3_address = allocator.malloc(1, std::iter::empty()).unwrap();
        let props_address = allocator.malloc(USIZE_SIZE + 2 * (USIZE_SIZE + VALUE_SIZE), std::iter::empty()).unwrap();
        let props_address2 = allocator.malloc(USIZE_SIZE + 2 * (USIZE_SIZE + VALUE_SIZE), std::iter::empty()).unwrap();
        let address = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let address2 = allocator.malloc(USIZE_SIZE, std::iter::empty()).unwrap();
        let tags_address = allocator.malloc(0, std::iter::empty()).unwrap();
        memory.copy_string("A", prop_address);
        memory.copy_string("B", prop1_address);
        memory.copy_string("B", prop2_address);
        memory.copy_string("C", prop3_address);
        memory.copy_t(&props_address, address);
        memory.copy_t(&props_address2, address2);
        memory.copy_t(&2usize, props_address);
        memory.copy_t_slice(&[(prop_address, Value::Nil), (prop1_address, Value::Nil)], props_address + USIZE_SIZE);
        memory.copy_t(&2usize, props_address2);
        memory.copy_t_slice(&[(prop2_address, Value::Nil), (prop3_address, Value::Nil)], props_address2 + USIZE_SIZE);
        let mut vm = VM::test_vm_with_memory_and_allocator(2, memory, allocator);
        vm.stack[0] = CompoundValue::SimpleValue(Value::Object {
            address,
            tags: tags_address,
        });
        vm.stack[1] = CompoundValue::SimpleValue(Value::Object {
            address: address2,
            tags: tags_address,
        });
        vm.execute_instruction(create_instruction(InstructionType::ObjectMerge))
            .unwrap();

        assert_eq!(vm.sp, 1);
        if let CompoundValue::SimpleValue(Value::Object {
                                              tags, address
                                          }) = vm.stack[0] {
            let address = *vm.memory.get_t::<usize>(address).unwrap();
            assert_eq!(
                Some(0),
                vm.allocator.borrow().get_allocated_space(tags)
            );
            assert_eq!(
                Some(USIZE_SIZE + 3 * (USIZE_SIZE + VALUE_SIZE)),
                vm.allocator.borrow().get_allocated_space(address)
            );
            let object_length = *vm.memory.get_t::<usize>(address).unwrap();
            assert_eq!(object_length, 3);
            let property = *vm.memory.get_t::<(usize, Value)>(address + USIZE_SIZE).unwrap();
            assert_eq!(property, (prop_address, Value::Nil));
            let property = *vm.memory.get_t::<(usize, Value)>(address + USIZE_SIZE + USIZE_SIZE + VALUE_SIZE).unwrap();
            assert_eq!(property, (prop1_address, Value::Nil));
            let property = *vm.memory.get_t::<(usize, Value)>(address + USIZE_SIZE + (USIZE_SIZE + VALUE_SIZE) * 2).unwrap();
            assert_eq!(property, (prop3_address, Value::Nil));
        } else {
            panic!("Invalid value {:?}", vm.stack[0]);
        }
    }
}
