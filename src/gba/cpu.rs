use core::fmt;
use std::fmt::write;

use super::arm::ArmInstruction;

const CPSR_Z: u32 = 0x60000000;
const CPSR_C: u32 = 0x20000000;
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

    pub fn run_instruction(&mut self, inst: u32, ram: &mut [u32; 128]) {
        let op = ArmInstruction::from(inst);

        match op {
            ArmInstruction::CMP(_, o) =>  {
                let operand2 = if o.i {
                    let rotate = o.operand >> 8 & 0xf;
                    ((o.operand & 0xff) << rotate) as u32
                }
                else {
                    self.registers[o.rd as usize]
                };

                let res = self.registers[o.rn as usize] - operand2;
                self.cpsr |= CPSR_C & (res >> 2);
                self.cpsr |= CPSR_Z & !res;
        }
            _ => todo!(),
        }

        self.registers[PC] += 4;
    }
}
