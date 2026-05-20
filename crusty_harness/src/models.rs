use std::collections::HashMap;

use crusty::{Cpu, memory::Memory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Test {
    initial: CpuState,
    #[serde(rename="final")]
    end: CpuState,
    transactions: Vec<Transaction>,
    opcode: u32,
    base_addr: u32,
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

#[derive(Debug)]
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
                x.memory.insert(addr_key, t.data);
            }
        }
        x
    }
}

impl Memory for TestMemory {
    fn read_word(&self, address: usize) -> Result<u32, crusty::memory::MemoryError> {
        let addr_key = address >> 2;
        let data = *self.memory.get(&addr_key).unwrap_or(&0);
        Ok(data)
    }

    fn read_halfword(&self, address: usize) -> Result<u32, crusty::memory::MemoryError> {
        let shift = address & 0b10;
        let data = self.read_word(address)?;
        let res = data >> (shift as u32 * 8);
        Ok(res)       
    }

    fn read_byte(&self, address: usize) -> Result<u32, crusty::memory::MemoryError> {
        let shift = address & 0b11;
        let data = self.read_word(address)?;
        let res = data >> (shift as u32 * 8);
        Ok(res)
    }

    fn read_byte_sign_ex(&self, address: usize) -> Result<u32, crusty::memory::MemoryError> {
        let res = self.read_byte(address)? as i32;
        Ok(((res << 24) >> 24) as u32) 
    }

    fn read_halfword_sign_ex(&self, address: usize) -> Result<u32, crusty::memory::MemoryError> {
        let res = self.read_halfword(address)? as i32;
        Ok(((res << 16) >> 16) as u32)
    }
}

