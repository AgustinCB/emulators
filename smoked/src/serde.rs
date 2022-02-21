use crate::allocator::Allocator;
use crate::cpu::{Location, NULL_VALUE, Value, STACK_MAX, VM, CompoundValue};
use crate::instruction::Instruction;
use crate::memory::Memory;
use std::cell::RefCell;
use std::cmp::min;
use std::mem::size_of;

const USIZE_SIZE: usize = std::mem::size_of::<usize>();

fn extract_usize(bytes: &[u8]) -> usize {
    *unsafe { (bytes.as_ptr() as *const usize).as_ref() }.unwrap()
}

fn extract_constants<I: Iterator<Item=u8>>(bytes: &mut I, memory: &[u8]) -> (Vec<usize>, Vec<Value>) {
    let mut constants = vec![];
    let mut sizes = vec![];
    let mut peakable = bytes.peekable();
    while peakable.peek().is_some() {
        let value = Value::from(&mut peakable);
        constants.push(value);
        match value {
            Value::String(address) | Value::Array { address, .. } |
                Value::Pointer(address) => {
                sizes.push(address);
            }
            Value::Object { address, tags, ..} => {
                sizes.push(address);
                let props_address = extract_usize(&memory[address..address + USIZE_SIZE]);
                sizes.push(props_address);
                sizes.push(tags);
            }
            _ => {}
        }
    }
    (sizes, constants)
}

#[macro_export]
macro_rules! serialize_type {
    ($bytes: ident, $value: expr, $type: ident) => {
        let p: &[u8] = unsafe {
            std::slice::from_raw_parts(&$value as *const $type as *const u8, size_of::<$type>())
        };
        $bytes.extend_from_slice(p);
    };
}

pub fn to_bytes(
    constants: &[Value],
    locations: &[Location],
    memory: &[u8],
    instructions: &[Instruction],
) -> Vec<u8> {
    let mut output = vec![];
    let mut upcodes = vec![];
    let mut constant_bytes = vec![];
    for i in instructions {
        let bs: Vec<u8> = i.clone().into();
        upcodes.extend_from_slice(&bs);
    }
    for c in constants {
        let bs: Vec<u8> = (*c).into();
        constant_bytes.extend_from_slice(&bs);
    }
    serialize_type!(output, constant_bytes.len(), usize);
    serialize_type!(output, memory.len(), usize);
    serialize_type!(output, locations.len(), usize);
    output.extend_from_slice(&constant_bytes);
    output.extend_from_slice(&memory);
    for _l in locations {
        serialize_type!(output, _l.address, usize);
        serialize_type!(output, _l.line, usize);
    }
    output.extend_from_slice(&upcodes);
    output
}

pub fn from_bytes(bytes: &[u8], stack_size: Option<usize>) -> VM {
    let constant_length = extract_usize(&bytes[0..USIZE_SIZE]);
    let memory_length = extract_usize(&bytes[USIZE_SIZE..USIZE_SIZE * 2]);
    let location_length = extract_usize(&bytes[USIZE_SIZE * 2..USIZE_SIZE * 3]);
    let memory_bytes = &bytes[USIZE_SIZE * 3 + constant_length
        ..USIZE_SIZE * 3 + constant_length + memory_length];
    let (addresses, constants) = extract_constants(
        &mut bytes[USIZE_SIZE * 3..USIZE_SIZE * 3 + constant_length].iter().cloned(), memory_bytes
    );
    let constants = constants.into_iter().map(|v| {
        CompoundValue::SimpleValue(v)
    }).collect();
    let mut sizes = vec![];
    let mut diffs = addresses;
    diffs.sort();
    diffs.push(memory_length);
    for (i, s) in diffs[1..].iter().enumerate() {
        sizes.push(s - diffs[i]);
    }
    let stack_size = stack_size.unwrap_or(memory_length);
    let memory = Memory::new(stack_size);
    memory.copy_u8_vector(memory_bytes, 0);
    let mut locations = vec![];
    for i in 0..location_length {
        locations.push(Location {
            address: extract_usize(
                &bytes[USIZE_SIZE * 3 + constant_length + memory_length + i * 2 * USIZE_SIZE
                    ..USIZE_SIZE * 3
                    + constant_length
                    + memory_length
                    + (i * 2 + 1) * USIZE_SIZE],
            ),
            line: extract_usize(
                &bytes[USIZE_SIZE * 3
                    + constant_length
                    + memory_length
                    + (i * 2 + 1) * USIZE_SIZE
                    ..USIZE_SIZE * 3
                    + constant_length
                    + memory_length
                    + (i * 2 + 2) * USIZE_SIZE],
            ),
        });
    }
    let bytes = &bytes
        [USIZE_SIZE * 3 + constant_length + memory_length + location_length * 2 * USIZE_SIZE..];
    let mut rom = vec![];
    let mut index = 0;
    while index < bytes.len() {
        let to = min(index + 17, bytes.len());
        let instruction = Instruction::from(&bytes[index..to]);
        index += instruction.size() as usize;
        rom.push(instruction);
    }
    let mut vm = VM {
        allocator: RefCell::new(Allocator::new_with_addresses(stack_size, &sizes).unwrap()),
        debug: false,
        frames: vec![],
        globals: Default::default(),
        sp: 0,
        stack: [NULL_VALUE; STACK_MAX],
        constants,
        locations,
        memory,
        rom,
    };
    vm.new_frame(0, 0);
    vm
}

#[cfg(test)]
mod tests {
    use crate::cpu::{Location, Value, CompoundValue};
    use crate::instruction::{Instruction, InstructionType};
    use crate::serde::{from_bytes, to_bytes};

    fn create_instruction(instruction_type: InstructionType) -> Instruction {
        Instruction {
            instruction_type,
            location: 0,
        }
    }

    #[test]
    fn it_should_serialize_a_vm() {
        let bytes = [
            78u8, 0, 0, 0, 0, 0, 0, 0, // Constant length
            8, 0, 0, 0, 0, 0, 0, 0, // Memory length
            1, 0, 0, 0, 0, 0, 0, 0, // Locations length
            0, // Nil value - 1
            1, 42, 0, 0, 0, 0, 0, 0, 0, // Integer value - 10
            2, 42, 42, 42, 42, // Float value - 15
            3, 1, // Bool value - 17
            4, 4, 0, 0, 0, 0, 0, 0, 0, // String value - 26
            5, 42, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0, // Function value - 43
            6, 2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, // Array value - 52
            7, 6, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, // Object value - 61
            0, 1, 2, 3, 4, 5, 6, 7, // Memory
            1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, // Locations
            0, 0, 0, 0, 0, 0, 0, 0, 0, // ROM
            1, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let got = to_bytes(&[
            Value::Nil, Value::Integer(42), Value::Float(0.00000000000015113662f32), Value::Bool(true),
            Value::String(4), Value::Function { arity: 42, ip: 42, uplifts: None, }, Value::Array { capacity: 2, address: 4},
            Value::Object { address: 6, tags: 6 },
        ],&[Location { address: 1, line: 1, }], &[0u8, 1, 2, 3, 4, 5, 6, 7],
            &[
                create_instruction(InstructionType::Return),
                create_instruction(InstructionType::Constant(42)),
                create_instruction(InstructionType::Plus),
                create_instruction(InstructionType::Minus),
                create_instruction(InstructionType::Mult),
            ]
        );
        assert_eq!(bytes.to_vec(), got);
    }

    #[test]
    fn it_should_deserialize_into_a_vm() {
        let bytes = [
            78u8, 0, 0, 0, 0, 0, 0, 0, // Constant length
            14, 0, 0, 0, 0, 0, 0, 0, // Memory length
            1, 0, 0, 0, 0, 0, 0, 0, // Locations length
            0, // Nil value - 1
            1, 42, 0, 0, 0, 0, 0, 0, 0, // Integer value - 10
            2, 42, 42, 42, 42, // Float value - 15
            3, 1, // Bool value - 17
            4, 4, 0, 0, 0, 0, 0, 0, 0, // String value - 26
            5, 42, 0, 0, 0, 0, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 0, // Function value - 43
            6, 2, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, // Array value - 52
            7, 6, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,// Object value - 69
            0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, // Memory
            1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, // Locations
            0, 0, 0, 0, 0, 0, 0, 0, 0, // ROM
            1, 42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0,
            0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let vm = from_bytes(bytes.as_ref(), None);
        assert_eq!(vm.constants.len(), 8);
        assert_eq!(&vm.constants[0], &CompoundValue::SimpleValue(Value::Nil));
        assert_eq!(&vm.constants[1], &CompoundValue::SimpleValue(Value::Integer(42)));
        assert_eq!(&vm.constants[2], &CompoundValue::SimpleValue(Value::Float(0.00000000000015113662f32)));
        assert_eq!(&vm.constants[3], &CompoundValue::SimpleValue(Value::Bool(true)));
        assert_eq!(&vm.constants[4], &CompoundValue::SimpleValue(Value::String(4)));
        assert_eq!(&vm.constants[5], &CompoundValue::SimpleValue(Value::Function { arity: 42, ip: 42, uplifts: None }));
        assert_eq!(
            &vm.constants[6],
            &CompoundValue::SimpleValue(Value::Array {
                capacity: 2,
                address: 4
            })
        );
        assert_eq!(&vm.constants[7], &CompoundValue::SimpleValue(Value::Object { address: 6, tags: 6 }));
        assert_eq!(vm.memory.get_capacity(), 14);
        assert_eq!(
            vm.memory.get_u8_vector(0, 14).unwrap(),
            &[0u8, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            &vm.locations,
            &[Location {
                address: 1,
                line: 1,
            }]
        );
        assert_eq!(
            &vm.rom,
            &[
                create_instruction(InstructionType::Return),
                create_instruction(InstructionType::Constant(42)),
                create_instruction(InstructionType::Plus),
                create_instruction(InstructionType::Minus),
                create_instruction(InstructionType::Mult),
            ]
        );
    }
}
