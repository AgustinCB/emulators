#![feature(bindings_after_at)]
#[macro_use] extern crate failure;
pub mod allocator;
pub mod cpu;
pub mod instruction;
pub mod memory;
pub mod serde;
