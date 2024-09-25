use core::fmt;

use super::arm::{decode_as_arm, ArmInstruction};
use super::{Conditional, CPSR_Z, CPSR_V, CPSR_N, CPSR_C};
use super::system::SystemMemory;

pub const PC: usize = 15;

#[derive(Debug)]
pub struct CPU {
    pub registers: [u32; 16],
    pub cpsr: u32,
    pub spsr: u32,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x68],
            cpsr: 0x1f,
            // TODO: Check if spsr is zero'd out at execution start
            spsr: 0x0,
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

    pub fn get_instruction_at_pc(&self) {

    }

    pub fn get_instruction_address(&self, address: u32) {

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

    pub fn run_instruction_v2(&mut self, inst: u32, ram: &mut SystemMemory) {
        self.registers[PC] += 4;

        let cond = Conditional::from(inst);
        if !cond.should_run(self.cpsr) {
            return;
        }

        let op = decode_as_arm(inst);
        op.run(self, ram);
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
                self.update_cpsr(res);
            },
            ArmInstruction::MOV(_, o) => {
                let operand2 = o.get_operand2(self.registers);
                self.registers[o.rd as usize] = operand2;
            },
            ArmInstruction::TEQ(_, o) => {
                let operand2 = o.get_operand2(self.registers);
                let res = self.registers[o.rn as usize] ^ operand2;
                self.update_cpsr(res);
            },
            ArmInstruction::ORR(_, o) => {
                let operand2 = o.get_operand2(self.registers);
                let res = self.registers[o.rn as usize] | operand2;
                if o.s {
                    self.update_cpsr(res);
                }
            },
            ArmInstruction::B(_, o) => {
                let offset = o.get_offset();
                let offset_abs: u32 = u32::try_from(offset.abs()).unwrap_or(0);

                if offset < 0 {
                    self.registers[PC] -= offset_abs;
                } else {
                    self.registers[PC] += offset_abs;
                }
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
                // for LDR
                if !o.p {
                    todo!()
                }
            },
            ArmInstruction::STR(_, o) => {
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

            },
            ArmInstruction::MRS(_, o) => {
                if o.is_cspr() {
                    self.registers[o.rd as usize] = self.cpsr;
                } else {
                    self.registers[o.rd as usize] = self.spsr;
                }
            },
            ArmInstruction::MSR(_, o) => {
                let operand = o.get_operand(self.registers);
                let mask: u32 = if o.is_bit_flag_only() {
                    0xf0000000
                } else {
                    0xffffffff
                };

                if o.is_cspr() {
                    self.cpsr = (self.cpsr & !mask) | (operand & mask)
                } else {
                    self.spsr = (self.spsr & !mask) | (operand & mask)
                }
            },
            _ => todo!(),
        }
    }
}
