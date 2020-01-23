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
    Nil,
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Integer(i) => i != 0,
            Value::Float(f) => f != 0.0,
            Value::Bool(b) => b,
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
    #[fail(display = "Invalid constant index {}", 0)]
    InvalidConstant(usize),
}

pub struct VM {
    ip: u64,
    sp: usize,
    stack: [Value; STACK_MAX],
    constants: Vec<Value>,
    memory: Vec<u8>,
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
    use super::{STACK_MAX, Value, VM, VMError};

    #[test]
    fn test_pop() -> Result<(), VMError> {
        let mut vm = VM {
            ip: 0,
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        let v = vm.pop()?;
        assert_eq!(v, Value::Integer(0));
        Ok(())
    }

    #[test]
    fn test_pop_on_empty_stack() {
        let mut vm = VM {
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        let v = vm.pop();
        assert_eq!(v, Err(VMError::EmptyStack));
    }

    #[test]
    fn test_push() -> Result<(), VMError> {
        let mut vm = VM {
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.push(Value::Integer(1))?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_push_on_stack() {
        let mut vm = VM {
            ip: 0,
            sp: STACK_MAX,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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

impl Cpu<u8, Instruction, VMError> for VM {
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
        let to = min(from + 9, self.memory.len());
        for i in from..to {
            res.push(self.memory[i]);
        }
        res
    }

    #[inline]
    fn can_run(&self, _: &Instruction) -> bool {
        true
    }

    #[inline]
    fn is_done(&self) -> bool {
        self.ip >= self.memory.len() as _
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
    use crate::instruction::Instruction;
    use failure::Error;
    use super::{STACK_MAX, Value, VM};

    #[test]
    fn test_constant() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: vec![Value::Integer(1)],
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Constant(0))?;
        assert_eq!(vm.stack[0], Value::Integer(1));
        Ok(())
    }

    #[test]
    fn test_add_integer() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Nil)?;
        assert_eq!(vm.stack[0], Value::Nil);
        Ok(())
    }

    #[test]
    fn test_true() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::True)?;
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_false() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 0,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::False)?;
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 1,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Equal)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_equals_diff() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::NotEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_not_equals_diff() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Greater)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_greater_greater() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::GreaterEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_greater_equals_greater() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::Less)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_less_greater() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.execute_instruction(&Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_less_equals_greater() -> Result<(), Error> {
        let mut vm = VM {
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
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
            ip: 0,
            sp: 2,
            stack: [Value::Integer(0); STACK_MAX],
            constants: Vec::new(),
            memory: Vec::new(),
        };
        vm.stack[1] = Value::Integer(-1);
        vm.execute_instruction(&Instruction::LessEqual)?;
        assert_eq!(vm.sp, 1);
        assert_eq!(vm.stack[0], Value::Bool(true));
        Ok(())
    }
}
