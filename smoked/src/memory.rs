use std::cell::RefCell;
use std::mem::size_of;

#[derive(Debug, Fail)]
pub enum MemoryError {
    #[fail(display = "Address {} is out of bounds", address)]
    WrongMemoryAddress { address: usize },
    #[fail(display = "Error fetching type from address")]
    ErrorFetchingFunctionFromMemory,
}

#[derive(Clone)]
pub struct Memory(RefCell<Vec<u8>>);

impl Memory {
    pub fn new(capacity: usize) -> Memory {
        let memory = RefCell::new(Vec::with_capacity(capacity));
        {
            let mut raw_memory = memory.borrow_mut();
            for _ in 0..capacity {
                raw_memory.push(0);
            }
        }
        Memory(memory)
    }

    pub(crate) fn get_t<T>(&self, address: usize) -> Result<&T, MemoryError> {
        let raw_data = self.get_u8_vector(address, size_of::<T>())?;
        let res = unsafe { (raw_data.as_ptr() as *const T).as_ref() }
            .ok_or(MemoryError::ErrorFetchingFunctionFromMemory)?;
        Ok(res)
    }

    pub fn copy_t<T>(&self, value: &T, address: usize) {
        let v: *const T = value;
        let p: &[u8] = unsafe { std::slice::from_raw_parts(v as *const u8, size_of::<T>()) };
        self.copy_u8_vector(p, address)
    }

    pub(crate) fn get_u8_vector(&self, address: usize, size: usize) -> Result<&[u8], MemoryError> {
        let memory: &[u8] = unsafe {
            std::slice::from_raw_parts(self.0.borrow()[address..].as_ptr(), size)
        };
        Ok(memory)
    }

    pub(crate) fn copy_u8_vector(&self, vector: &[u8], address: usize) {
        let memory: &mut [u8] = unsafe {
            std::slice::from_raw_parts_mut(
                self.0.borrow_mut()[address..].as_mut_ptr(),
                vector.len(),
            )
        };
        memory.copy_from_slice(vector);
    }

    pub(crate) fn get_string(&self, address: usize, size: usize) -> Result<&str, MemoryError> {
        let bytes = self.get_u8_vector(address, size)?;
        Ok(std::str::from_utf8(bytes).unwrap())
    }
}

#[cfg(test)]
impl Memory {
    pub(crate) fn copy_string(&self, value: &str, address: usize) {
        let bs = value.as_bytes();
        self.copy_u8_vector(bs, address)
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;

    #[test]
    fn it_should_copy_a_u8_aray() {
        let data = &[1u8, 1, 1, 1, 1, 1, 1, 1];
        let memory = Memory::new(12);
        memory.copy_u8_vector(data, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(&memory.0.borrow()[1..9], &[1u8, 1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(memory.0.borrow()[10], 0);
    }

    #[test]
    fn it_should_copy_a_type() {
        let memory = Memory::new(3);
        memory.copy_t(&true, 1);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 1);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_get_a_type() {
        let memory = Memory::new(3);
        memory.0.borrow_mut()[1] = 1;
        let result: bool = *memory.get_t(1).unwrap();
        assert_eq!(result, true);
        assert_eq!(memory.0.borrow()[0], 0);
        assert_eq!(memory.0.borrow()[1], 1);
        assert_eq!(memory.0.borrow()[2], 0);
    }

    #[test]
    fn it_should_be_able_to_store_a_string() {
        let s = String::from("42");
        let memory = Memory::new(10);
        memory.copy_string(&s, 0);
        let result = memory.get_string(0, s.as_bytes().len()).unwrap();
        assert_eq!(result, &s);
    }
}
