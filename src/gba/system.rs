use core::fmt;

const KILOBYTE: usize = 1024;
const WORD: u32 = 0xffffffff;
const HALFWORD: u32 = 0xffff;
const BYTE: u32 = 0xff;

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
    pub fn copy_bios(&mut self, bios: Vec<u32>) {
        self.system_rom = bios;
    }
}


impl SystemMemory {
    pub fn write_word(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, WORD)?;
        Ok(())
    }

    pub fn write_halfword(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, HALFWORD)?;
        Ok(())
    }

    pub fn write_byte(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, BYTE)?;
        Ok(())
    }

    fn write_with_mask(&mut self, address: usize, block: u32, mask: u32) -> Result<(), MemoryError> {
        let i = (address & 0xfffff) >> 2;
        let shift = (address & 0x3) as u32;

        let mut data = self.read_from_mem(address)?;
        data = (data & !(mask << shift)) | ((block & mask) << shift);

        let ram: &mut Vec<u32> = self.memory_map(address)?;
        if i > ram.len() {
            Err(MemoryError::OutOfBounds(address))
        } else {
            ram[i] = data;
            Ok(())
        }
    }

    pub fn read_word(&mut self, address: usize) -> Result<u32, MemoryError> {
        let res = self.read_from_mem(address)?;
        Ok(res)
    }

    pub fn read_halfword(&mut self, address: usize) -> Result<u32, MemoryError> {
        let res = self.read_from_mem(address)?;
        let shift = address & 0b10;
        // TODO: check that address is halfword aligned, error otherwise?
        Ok(res >> (shift * 4))
    }

    pub fn read_byte(&mut self, address: usize) -> Result<u32, MemoryError> {
        let res = self.read_from_mem(address)?;
        let shift = address & 0b11;
        Ok(res >> (shift * 4))
    }

    pub fn read_from_mem(&mut self, address: usize) -> Result<u32, MemoryError> {
        let ram: &Vec<u32> = self.memory_map(address)?;
        let mem_address = (address & 0xfffff) >> 2;
        if mem_address > ram.len() {
            Err(MemoryError::OutOfBounds(mem_address))
        } else {
            Ok(ram[mem_address])
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

