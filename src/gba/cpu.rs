use core::fmt;

use super::arm::{ArmInstruction, Conditional};
use super::system::SystemMemory;

pub const CPSR_Z: u32 = 0x60000000;
pub const CPSR_C: u32 = 0x20000000;
const PC: usize = 15;

#[derive(Debug)]
pub struct CPU {
    registers: [u32; 16],
    cpsr: u32,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x68],
            cpsr: 0x1f,
        }
    }
}

impl fmt::Display for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in (0..16).step_by(4) {
            write!(f, "r{}\t{:#08x}\t", i, self.registers[i])?;
            write!(f, "r{}\t{:#08x}\t", i + 1, self.registers[i + 1])?;
            write!(f, "r{}\t{:#08x}\t", i + 2, self.registers[i + 2])?;
            write!(f, "r{}\t{:#08x}\n", i + 3, self.registers[i + 3])?;
        }
        write!(f, "cpsr: {:#8x}\n", self.cpsr)
    }
}

impl CPU {
    // TODO: Make these mutable pointers?

    // Stack Pointer
    pub fn sp(&self) -> u32 {
        self.registers[13]
    }

    // Link Register
    pub fn lr(&self) -> u32 {
        self.registers[14]
    }

    // Program Counter
    pub fn pc(&self) -> u32 {
        self.registers[PC]
    }
    pub fn set_pc(&mut self, pc: u32) {
        self.registers[PC] = pc;
    }

    pub fn run_instruction(&mut self, inst: u32, ram: &mut SystemMemory) {
        let cond = Conditional::from(inst);
        let op = ArmInstruction::from(inst);
        self.registers[PC] += 4;

        if !cond.should_run(self.cpsr) {
            return;
        }

        match op {
            ArmInstruction::CMP(_, o) =>  {
                let operand2 = o.get_operand2(self.registers);
                let res = self.registers[o.rn as usize] - operand2;
                self.cpsr |= CPSR_C & (res >> 2);
                self.cpsr |= CPSR_Z & !res;
            },
            ArmInstruction::MOV(_, o) => {
                let operand2 = o.get_operand2(self.registers);
                self.registers[o.rd as usize] = operand2;
            },
            ArmInstruction::LDR(_, o) => {
                // TODO: add write back check somewhere
                let offset = o.get_offset(self.registers);
                let mut tfx_add = offset;
                tfx_add >>= 2;

                if o.p {
                    if o.u {
                        tfx_add += self.registers[o.rn as usize];
                    } else {
                        tfx_add -= self.registers[o.rn as usize];
                    }
                }

                let block_from_mem = match ram.read_from_mem(tfx_add as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        panic!()
                    },
                };

                self.registers[o.rd as usize] = if o.b {
                    block_from_mem
                } else {
                    block_from_mem & 0xff
                };

                // NOTE: for L i don't think this matters
                if !o.p {

                }
            },
            _ => todo!(),
        }
    }
}
