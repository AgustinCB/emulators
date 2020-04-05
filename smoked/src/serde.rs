use crate::allocator::Allocator;
use crate::cpu::{STACK_MAX, VM, Value};
use crate::instruction::{Instruction as VMInstruction};
use crate::memory::Memory;
use cpu::Instruction;
use std::cell::RefCell;
use std::cmp::min;

const I64_SIZE: usize = std::mem::size_of::<i64>();
const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const F32_SIZE: usize = std::mem::size_of::<f32>();

fn extract_usize(bytes: &[u8]) -> usize {
    *unsafe {
        (bytes.as_ptr() as *const usize).as_ref()
    }.unwrap()
}

fn extract_constants(bytes: &[u8], size: usize) -> (Vec<usize>, Vec<Value>) {
    let mut constants = vec![];
    let mut index = 0;
    let mut sizes = vec![];
    let mut last_address = 0;
    while index < size {
        index += 1;
        match bytes[index-1] {
            0 => constants.push(Value::Nil),
            1 => {
                let integer = *unsafe {
                    (bytes[index..index+I64_SIZE].as_ptr() as *const i64).as_ref()
                }.unwrap();
                index += I64_SIZE;
                constants.push(Value::Integer(integer));
            },
            2 => {
                let float = *unsafe {
                    (bytes[index..index+F32_SIZE].as_ptr() as *const f32).as_ref()
                }.unwrap();
                index += F32_SIZE;
                constants.push(Value::Float(float));
            },
            3 => {
                let bool = bytes[index] != 0;
                index += 1;
                constants.push(Value::Bool(bool))
            },
            4 => {
                let length = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                constants.push(Value::String(last_address));
                sizes.push(length);
                last_address += length;
            },
            5 => {
                let ip = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                let arity = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                constants.push(Value::Function { ip, arity, });
            },
            6 => {
                let capacity = extract_usize(&bytes[index..index+USIZE_SIZE]);
                let address = last_address;
                index += USIZE_SIZE;
                constants.push(Value::Array { capacity, address });
                sizes.push(capacity);
                last_address += capacity;
            },
            7 => {
                let capacity = extract_usize(&bytes[index..index+USIZE_SIZE]);
                let address = last_address;
                index += USIZE_SIZE;
                constants.push(Value::Object { address });
                sizes.push(capacity);
                last_address += capacity;
            },
            _ => panic!("Invalid value type")
        }
    }
    (sizes, constants)
}

impl From<&[u8]> for VM {
    fn from(bytes: &[u8]) -> Self {
        let constant_length = extract_usize(&bytes[0..USIZE_SIZE]);
        let memory_length = extract_usize(&bytes[USIZE_SIZE..USIZE_SIZE*2]);
        let (addresses, constants) = extract_constants(
            &bytes[USIZE_SIZE*2..USIZE_SIZE*2 + constant_length], constant_length
        );
        let memory = Memory::new(memory_length);
        memory.copy_u8_vector(
            &bytes[USIZE_SIZE * 2 + constant_length..USIZE_SIZE * 2 + constant_length + memory_length],
            0
        );
        let bytes = &bytes[USIZE_SIZE * 2 + constant_length + memory_length..];
        let mut rom = vec![];
        let mut index = 0;
        while index < bytes.len() {
            let to = min(index+9, bytes.len());
            let instruction = VMInstruction::from(bytes[index..to].to_vec());
            index += instruction.size().unwrap() as usize;
            rom.push(instruction);
        }
        let mut vm = VM {
            allocator: RefCell::new(
                Allocator::new_with_addresses(memory_length, &addresses).unwrap()
            ),
            frames: vec![],
            globals: Default::default(),
            sp: 0,
            stack: [Value::Nil; STACK_MAX],
            constants,
            memory,
            rom,
        };
        vm.new_frame(0, 0);
        vm
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::{VM, Value};
    use crate::instruction::Instruction;

    #[test]
    fn it_should_deserialize_into_a_vm() {
        let bytes = [
            61u8, 0, 0, 0, 0, 0, 0, 0, // Constant length
            8, 0, 0, 0, 0, 0, 0, 0, // Memory length
            0, // Nil value
            1, 42, 0, 0, 0, 0, 0, 0, 0, // Integer value
            2, 42, 42, 42, 42, // Float value
            3, 1, // Bool value
            4, 4, 0, 0, 0, 0, 0, 0, 0, // String value
            5, 42, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, // Function value
            6, 2, 0, 0, 0, 0, 0, 0, 0, // Array value
            7, 2, 0, 0, 0, 0, 0, 0, 0, // Object value
            0, 1, 2, 3, 4, 5, 6, 7, // Memory
            0, 1, 42, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4 // ROM
        ];
        let vm = VM::from(bytes.as_ref());
        assert_eq!(vm.constants.len(), 8);
        assert_eq!(&vm.constants[0], &Value::Nil);
        assert_eq!(&vm.constants[1], &Value::Integer(42));
        assert_eq!(&vm.constants[2], &Value::Float(0.00000000000015113662f32));
        assert_eq!(&vm.constants[3], &Value::Bool(true));
        assert_eq!(&vm.constants[4], &Value::String(0));
        assert_eq!(&vm.constants[5], &Value::Function { arity: 42, ip: 42 });
        assert_eq!(&vm.constants[6], &Value::Array { capacity: 2, address: 4 });
        assert_eq!(&vm.constants[7], &Value::Object { address: 6 });
        assert_eq!(vm.memory.get_capacity(), 8);
        assert_eq!(vm.memory.get_u8_vector(0, 8).unwrap(), &[0u8, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&vm.rom, &[
            Instruction::Return, Instruction::Constant(42), Instruction::Plus, Instruction::Minus,
            Instruction::Mult
        ]);
    }
}
