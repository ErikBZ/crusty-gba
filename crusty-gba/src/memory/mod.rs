use std::fmt::{Display, Formatter, Error};

pub trait Memory {
    fn read_word(&self, address: usize) -> Result<u32, MemoryError>;
    fn read_halfword(&self, address: usize) -> Result<u32, MemoryError>;
    fn read_halfword_sign_ex(&self, address: usize) -> Result<u32, MemoryError>;
    fn read_byte(&self, address: usize) -> Result<u32, MemoryError>;
    fn read_byte_sign_ex(&self, address: usize) -> Result<u32, MemoryError>;
    fn write_word(&mut self, address: usize, block: u32) -> Result<(), MemoryError>;
    fn write_halfword(&mut self, address: usize, block: u32) -> Result<(), MemoryError>;
    fn write_byte(&mut self, address: usize, block: u32) -> Result<(), MemoryError>;
}

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryError {
    OutOfBounds(usize, usize),
    MapNotFound(usize),
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::OutOfBounds(a, b) => write!(
                f,
                "Memory Address is out of bounds: {:#010x} index: {:#010x}",
                a, b
            ),
            Self::MapNotFound(a) => write!(f, "Memory Mapping not found for address: {:#010x}", a),
        }
    }
}
