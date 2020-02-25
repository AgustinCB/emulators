use std::collections::HashMap;
use std::mem::size_of;

#[derive(Debug, Fail)]
pub enum AllocatorError {
    #[fail(display = "Not enough memory to allocate {}", intended)]
    NotEnoughMemory { intended: usize },
    #[fail(display = "Address {} not allocated", address)]
    AddressNotAllocated { address: usize },
    #[fail(display = "Trying to free address {} already freed", address)]
    AddressAlreadyFreed { address: usize },
}

struct FreeChunks {
    free_chunks: Vec<(usize, usize)>,
}

impl FreeChunks {
    fn new(capacity: usize) -> FreeChunks {
        FreeChunks {
            free_chunks: vec![(0, capacity)],
        }
    }

    fn remove(&mut self, index: usize) {
        self.free_chunks.remove(index);
    }

    fn insert(&mut self, item: (usize, usize)) -> Result<(), AllocatorError> {
        match self
            .free_chunks
            .binary_search_by(|(f, t)| (item.1 - item.0).cmp(&(t - f)))
        {
            Ok(_) => return Err(AllocatorError::AddressAlreadyFreed { address: item.0 }.into()),
            Err(pos) => {
                self.free_chunks.insert(pos, item);
            }
        };
        Ok(())
    }

    fn get_adjacent_chunk(&self, from: usize, to: usize) -> Option<(usize, (usize, usize))> {
        self.free_chunks
            .iter()
            .cloned()
            .enumerate()
            .find(|(_, (f, t))| *f == to || from == *t)
            .map(|(i, (f, t))| (i, (f, t)))
    }

    fn find_suitable_chunk(&self, size: usize) -> Option<(usize, (usize, usize))> {
        self.free_chunks
            .iter()
            .cloned()
            .rev()
            .enumerate()
            .find(|(_, (from, to))| (*to - *from) >= size)
            .map(|(i, (f, t))| (self.free_chunks.len() - i - 1, (f, t)))
    }

    fn available_memory(&self) -> usize {
        self.free_chunks
            .iter()
            .cloned()
            .map(|(from, to)| to - from)
            .sum()
    }
}

pub struct Allocator {
    free_chunks: FreeChunks,
    allocated_spaces: HashMap<usize, usize>,
}

impl Allocator {
    pub fn new(capacity: usize) -> Allocator {
        Allocator {
            allocated_spaces: HashMap::new(),
            free_chunks: FreeChunks::new(capacity),
        }
    }

    pub fn get_allocated_space(&self, address: usize) -> Option<usize> {
        self.allocated_spaces.get(&address).cloned()
    }

    pub fn malloc_t<T>(&mut self) -> Result<usize, AllocatorError> {
        self.malloc(size_of::<T>())
    }

    pub fn malloc(&mut self, size: usize) -> Result<usize, AllocatorError> {
        let free_memory = self.free_chunks.available_memory();
        if size > free_memory {
            Err(AllocatorError::NotEnoughMemory {
                intended: size,
            })
        } else {
            let space = self.free_chunks.find_suitable_chunk(size);
            match space {
                None => Err(AllocatorError::NotEnoughMemory {
                    intended: size,
                }),
                Some((index, (from, to))) => {
                    self.free_chunks.remove(index);
                    if from + size < to {
                        self.free_chunks.insert((from + size, to))?;
                    }
                    self.allocated_spaces.insert(from, size);
                    Ok(from)
                }
            }
        }
    }

    pub fn free(&mut self, address: usize) -> Result<(), AllocatorError> {
        match self.allocated_spaces.get(&address).cloned() {
            Some(space) => {
                self.add_free_space(address, address + space)?;
                self.allocated_spaces.remove(&address);
                Ok(())
            }
            None => Err(AllocatorError::AddressNotAllocated { address }),
        }
    }

    fn add_free_space(&mut self, from: usize, to: usize) -> Result<(), AllocatorError> {
        let adjacent = self.free_chunks.get_adjacent_chunk(from, to);
        match adjacent {
            Some((i, (f, t))) => {
                self.free_chunks.remove(i);
                self.free_chunks
                    .insert(if f == to { (from, t) } else { (f, to) })
            }
            None => self.free_chunks.insert((from, to)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::allocator::Allocator;

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: NotEnoughMemory { intended: 3 }"
    )]
    fn it_should_error_if_trying_to_allocate_more_space_than_memory_capacity() {
        let mut allocator = Allocator::new(2);
        allocator.malloc(3).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: NotEnoughMemory { intended: 1 }"
    )]
    fn it_should_error_if_trying_to_allocate_more_space_than_available() {
        let mut allocator = Allocator::new(2);
        allocator.malloc(2).unwrap();
        allocator.malloc(1).unwrap();
    }

    #[test]
    fn it_should_return_the_first_address_available() {
        let mut allocator = Allocator::new(2);
        assert_eq!(allocator.malloc(1).unwrap(), 0);
        assert_eq!(allocator.malloc(1).unwrap(), 1);
    }

    #[test]
    fn it_should_correctly_free_memory() {
        let mut allocator = Allocator::new(2);
        let address = allocator.malloc(2).unwrap();
        allocator.free(address).unwrap();
        assert_eq!(allocator.malloc(2).unwrap(), 0);
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: AddressNotAllocated { address: 1 }"
    )]
    fn it_should_fail_when_freeing_unallocated_space() {
        let mut allocator = Allocator::new(2);
        allocator.malloc(2).unwrap();
        allocator.free(1).unwrap();
    }

    #[test]
    fn it_should_defragment_memory() {
        let mut allocator = Allocator::new(5);
        let address1 = allocator.malloc(2).unwrap();
        let address2 = allocator.malloc(2).unwrap();
        allocator.free(address1).unwrap();
        allocator.free(address2).unwrap();
        allocator.malloc(4).unwrap();
        allocator.malloc(1).unwrap();
    }

    #[test]
    fn it_should_allocate_from_the_smallest_chunk_possible() {
        let mut allocator = Allocator::new(5);
        let address1 = allocator.malloc(2).unwrap();
        let address2 = allocator.malloc(2).unwrap();
        allocator.free(address1).unwrap();
        allocator.free(address2).unwrap();
        allocator.malloc(1).unwrap();
        allocator.malloc(4).unwrap();
    }
}
