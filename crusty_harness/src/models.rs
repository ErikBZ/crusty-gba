use std::collections::HashMap;

use crusty::{Cpu, memory::Memory, memory::MemoryError};
use serde::{Deserialize, Serialize};

const WORD: u32 = 0xffffffff;
const HALFWORD: u32 = 0xffff;
const BYTE: u32 = 0xff;

#[derive(Debug, Deserialize)]
pub struct Test {
    initial: CpuState,
    #[serde(rename="final")]
    end: CpuState,
    transactions: Vec<Transaction>,
    opcode: u32,
    base_addr: u32,
}

pub async fn run_test(t: Test, idx: usize) -> Result<(u32, usize), (bool, usize)> {
    if t.opcode & 0x1 == 1 {
        Err((false, idx))
    } else {
        Ok((t.opcode, idx))
    }
}

#[derive(Debug, Deserialize)]
pub struct CpuState {
    #[serde(rename="R")]
    registers: [u32; 16],
    #[serde(rename="R_fiq")]
    r_fiq: [u32; 7],
    #[serde(rename="R_svc")]
    r_svc: [u32; 2],
    #[serde(rename="R_abt")]
    r_abt: [u32; 2],
    #[serde(rename="R_irq")]
    r_irq: [u32; 2],
    #[serde(rename="R_und")]
    r_und: [u32; 2],
    #[serde(rename="CPSR")]
    cpsr: u32,
    #[serde(rename="SPSR")]
    spsr: [u32; 5],
    pipeline: [usize; 2],
    access: u32,
}

impl From<CpuState> for Cpu {
    fn from(value: CpuState) -> Self {
        Cpu {
            registers: value.registers,
            fiq_banked_gen_regs: value.r_fiq,
            svc_banked_regs: value.r_svc,
            abt_banked_regs: value.r_abt,
            irq_banked_regs: value.r_irq,
            und_banked_regs: value.r_und,
            cpsr: value.cpsr,
            psr: value.spsr,
            inst_addr: value.pipeline[0],
            cycles: 0,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    kind: u32,
    size: u32,
    addr: usize,
    data: u32,
    cycle: u32,
    access: u32,
}

#[derive(Debug, Serialize)]
pub struct TestMemory {
    memory: HashMap<usize, u32>
}

impl TestMemory {
    pub fn new(transactions: Vec<Transaction>) -> TestMemory {
        let mut x = Self {
            memory: HashMap::new()
        };
        for t in transactions {
            if t.kind == 1 || t.kind == 0 {
                let addr_key = t.addr;
                if x.memory.contains_key(&addr_key) {
                    println!("Memory already has this item!")
                }

                if t.size == 4 {
                    let _ = x.write_word(t.addr, t.data);
                } else if t.size == 2 {
                    let _ = x.write_halfword(t.addr, t.data);
                } else if t.size == 1 {
                    let _ = x.write_byte(t.addr, t.data);
                } else {
                    println!("Size was not 4, 2, or 1");
                };
            }
        }
        x
    }

    // NOTE: I don't actually know is this is how things are suppose to get checked
    pub fn apply_write_transactions(&mut self, transactions: Vec<Transaction>) {
        for t in transactions {
            if t.kind == 2 {
                if t.size == 4 {
                    let _ = self.write_word(t.addr, t.data);
                } else if t.size == 2 {
                    let _ = self.write_halfword(t.addr, t.data);
                } else if t.size == 1 {
                    let _ = self.write_byte(t.addr, t.data);
                } else {
                    println!("Size was not 4, 2, or 1");
                };
            }
        }
    }

    fn write_with_mask(&mut self, address: usize, block: u32, mask: u32) -> Result<(), crusty::memory::MemoryError> {
        let i = (address & 0xffffff) >> 2;
        let shift = (address & 0x3) * 8;
        let old_data = self.read_word(address)?;
        let new_data = (old_data & !(mask << shift)) | ((block & mask) << shift);

        self.memory.entry(i)
            .and_modify(|k| *k = new_data)
            .or_insert(new_data);
        Ok(())
    }
}

impl Memory for TestMemory {
    fn write_word(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, WORD)?;
        Ok(())
    }

    fn write_halfword(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, HALFWORD)?;
        Ok(())
    }

    fn write_byte(&mut self, address: usize, block: u32) -> Result<(), MemoryError> {
        self.write_with_mask(address, block, BYTE)?;
        Ok(())
    }

    fn read_word(&self, address: usize) -> Result<u32, MemoryError> {
        let addr_key = address >> 2;
        let data = *self.memory.get(&addr_key).unwrap_or(&0);
        Ok(data)
    }

    fn read_halfword(&self, address: usize) -> Result<u32, MemoryError> {
        let shift = address & 0b10;
        let data = self.read_word(address)?;
        let res = data >> (shift as u32 * 8);
        Ok(res)       
    }

    fn read_byte(&self, address: usize) -> Result<u32, MemoryError> {
        let shift = address & 0b11;
        let data = self.read_word(address)?;
        let res = data >> (shift as u32 * 8);
        Ok(res)
    }

    fn read_byte_sign_ex(&self, address: usize) -> Result<u32, MemoryError> {
        let res = self.read_byte(address)? as i32;
        Ok(((res << 24) >> 24) as u32) 
    }

    fn read_halfword_sign_ex(&self, address: usize) -> Result<u32, MemoryError> {
        let res = self.read_halfword(address)? as i32;
        Ok(((res << 16) >> 16) as u32)
    }
}

