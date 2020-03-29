use crate::cpu::{STACK_MAX, VM, Value};
use crate::memory::Memory;
use crate::allocator::Allocator;
use std::cell::RefCell;

const I64_SIZE: usize = std::mem::size_of::<i64>();
const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const F32_SIZE: usize = std::mem::size_of::<f32>();

fn extract_usize(bytes: &[u8]) -> usize {
    println!("{:?}", bytes);
    *unsafe {
        (bytes.as_ptr() as *const usize).as_ref()
    }.unwrap()
}

fn extract_constants(bytes: &[u8], size: usize) -> Vec<Value> {
    let mut constants = vec![];
    let mut index = 0;
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
                let address = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                constants.push(Value::String(address));
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
                index += USIZE_SIZE;
                let address = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                constants.push(Value::Array { capacity, address });
            },
            7 => {
                let address = extract_usize(&bytes[index..index+USIZE_SIZE]);
                index += USIZE_SIZE;
                constants.push(Value::Object { address });
            },
            _ => panic!("Invalid value type")
        }
    }
    constants
}

impl From<&[u8]> for VM {
    fn from(bytes: &[u8]) -> Self {
        let constant_length = extract_usize(&bytes[0..USIZE_SIZE]);
        let memory_length = extract_usize(&bytes[USIZE_SIZE..USIZE_SIZE*2]);
        let constants = extract_constants(
            &bytes[USIZE_SIZE*2..USIZE_SIZE*2 + constant_length], constant_length
        );
        let memory = Memory::new(memory_length);
        memory.copy_u8_vector(
            &bytes[USIZE_SIZE * 2 + constant_length..USIZE_SIZE * 2 + constant_length + memory_length],
            0
        );
        let rom = bytes[USIZE_SIZE * 2 + constant_length + memory_length..].to_vec();
        let mut vm = VM {
            allocator: RefCell::new(Allocator::new(memory_length)),
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

    #[test]
    fn it_should_deserialize_into_a_vm() {
        let bytes = [
            69u8, 0, 0, 0, 0, 0, 0, 0, // Constant length
            4, 0, 0, 0, 0, 0, 0, 0, // Memory length
            0, // Nil value
            1, 42, 0, 0, 0, 0, 0, 0, 0, // Integer value
            2, 42, 42, 42, 42, // Float value
            3, 1, // Bool value
            4, 42, 0, 0, 0, 0, 0, 0, 0, // String value
            5, 42, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, // Function value
            6, 42, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, // Array value
            7, 42, 0, 0, 0, 0, 0, 0, 0, // Object value
            0, 1, 2, 3, // Memory
            42, 43, 44, 45 // ROM
        ];
        let vm = VM::from(bytes.as_ref());
        assert_eq!(vm.constants.len(), 8);
        assert_eq!(&vm.constants[0], &Value::Nil);
        assert_eq!(&vm.constants[1], &Value::Integer(42));
        assert_eq!(&vm.constants[2], &Value::Float(0.00000000000015113662f32));
        assert_eq!(&vm.constants[3], &Value::Bool(true));
        assert_eq!(&vm.constants[4], &Value::String(42));
        assert_eq!(&vm.constants[5], &Value::Function { arity: 42, ip: 42 });
        assert_eq!(&vm.constants[6], &Value::Array { capacity: 42, address: 42 });
        assert_eq!(&vm.constants[7], &Value::Object { address: 42 });
        assert_eq!(vm.memory.get_capacity(), 4);
        assert_eq!(vm.memory.get_u8_vector(0, 4).unwrap(), &[0u8, 1, 2, 3]);
        assert_eq!(&vm.rom, &[42u8, 43, 44, 45]);
    }
}
