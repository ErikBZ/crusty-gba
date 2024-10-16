use core::fmt;

use super::arm::decode_as_arm;
use super::thumb::decode_as_thumb;
use super::{Conditional, CPSR_Z, CPSR_V, CPSR_N, CPSR_C, CPSR_T};
use super::system::SystemMemory;

pub const PC: usize = 15;
pub const LR: usize = 14;
pub const SP: usize = 13;

#[derive(Debug)]
pub struct CPU {
    pub registers: [u32; 16],
    pub cpsr: u32,
    pub spsr: u32,
    // Should one of these be the addr and the other the value?
    pub execute: u32,
    pub decode: u32,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x68],
            cpsr: 0x1f,
            // TODO: Check if spsr is zero'd out at execution start
            spsr: 0x0,
            // NOTE: This instruction ANDs the r0 with r0 doing nothing
            execute: 0x0,
            decode: 0x0,
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
    // Program Counter
    pub fn pc(&self) -> u32 {
        self.registers[PC]
    }

    pub fn get_instruction_at_pc(&self) {
        todo!()
    }

    pub fn get_instruction_address(&self, address: u32) {
        todo!()
    }

    pub fn update_cpsr(&mut self, res: u32) {
        let zero = if res == 0 {
            CPSR_Z
        } else {
            0
        };

        self.cpsr &= 0x2fffffff;
        self.cpsr |= CPSR_C & (res >> 2);
        self.cpsr |= zero;
    }

    pub fn update_thumb(&mut self, is_thumb: bool) {
        if is_thumb {
            self.cpsr |= CPSR_T;
        } else {
            self.cpsr &= !CPSR_T;
        }
    }

    pub fn is_thumb_mode(&self) -> bool {
        self.cpsr & CPSR_T == CPSR_T
    }

    pub fn tick(&mut self, ram: &mut SystemMemory) {
        let inst = self.decode;
        self.decode = match ram.read_from_mem(self.pc() as usize) {
            Ok(i) => i,
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        self.registers[PC] += if !self.is_thumb_mode() {
            4
        } else {
            2
        };

        let op = if !self.is_thumb_mode() {
            let cond = Conditional::from(inst);
            if !cond.should_run(self.cpsr) {
                return;
            }
            decode_as_arm(inst)
        } else {
            let inst = if self.pc() % 4 == 0 {
                inst & 0xffff
            } else {
                inst >> 16
            };

            decode_as_thumb(inst)
        };

        op.run(self, ram);
    }
}
