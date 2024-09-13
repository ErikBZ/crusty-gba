use core::fmt;

use strum::additional_attributes;

const KILOBYTE: usize = 1024;

#[derive(Debug, PartialEq, Clone)]
pub enum MemoryError {
    OutOfBounds(usize),
    MapNotFound(usize),
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OutOfBounds(a) => write!(f, "Memory Address is out of bounds: {:#08x}", a),
            Self::MapNotFound(a) => write!(f, "Memory Mapping not found for address: {:#08x}", a),
        }
    } 
}

pub struct SystemMemory {
    system_rom: Vec<u32>,
    ewram: Vec<u32>,
    iwram: Vec<u32>,
    io_ram: Vec<u32>,
    pal_ram: Vec<u32>,
    vram: Vec<u32>,
    oam: Vec<u32>,
    // TODO: do this later
    pak_rom: Vec<u32>,
    cart_ram: Vec<u32>,
}

impl Default for SystemMemory {
    fn default() -> Self {
        Self {
            system_rom: vec![0; 16 * KILOBYTE],
            ewram: vec![0; 256 * KILOBYTE],
            iwram: vec![0; 32 * KILOBYTE],
            io_ram: vec![0; 1 * KILOBYTE],
            pal_ram: vec![0; 1 * KILOBYTE],
            vram: vec![0; 96 * KILOBYTE],
            oam: vec![0; 1 * KILOBYTE],
            pak_rom: vec![0; 16 * 1],
            cart_ram: vec![0; 16 * 1],
        }
    }
}

impl SystemMemory {
    pub fn write_to_mem(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        let ram: &mut Vec<u32> = self.memory_map(address)?;
        if address > ram.len() {
            Err(MemoryError::OutOfBounds(address))
        } else {
            ram[address] = block;
            Ok(())
        }
    }

    pub fn read_from_mem(&mut self, address: usize) -> Result<u32, MemoryError> {
        let ram: &Vec<u32> = self.memory_map(address)?;
        if address > ram.len() {
            Err(MemoryError::OutOfBounds(address))
        } else {
            Ok(ram[address])
        }
    }
    
    // deal with lifetimes later
    fn memory_map(&mut self, address: usize) -> Result<&mut Vec<u32>, MemoryError> {
        let mem_type = address >> 24;
        match mem_type {
            0x0 => Ok(&mut self.system_rom),
            0x2 => Ok(&mut self.ewram),
            0x3 => Ok(&mut self.iwram),
            0x4 => Ok(&mut self.io_ram),
            0x5 => Ok(&mut self.pal_ram),
            0x6 => Ok(&mut self.vram),
            0x7 => Ok(&mut self.oam),
            0x8 => Ok(&mut self.pak_rom),
            0xe => Ok(&mut self.cart_ram),
            _ => Err(MemoryError::MapNotFound(address))
        }
    }
}

