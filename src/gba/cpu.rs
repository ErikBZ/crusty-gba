use core::fmt;

use super::arm::decode_as_arm;
use super::thumb::decode_as_thumb;
use super::{is_signed, Conditional, CPSR_C, CPSR_N, CPSR_T, CPSR_V, CPSR_Z};
use super::system::SystemMemory;

pub const PC: usize = 15;
pub const LR: usize = 14;
pub const SP: usize = 13;

#[derive(Debug, PartialEq, Eq)]
pub enum CpuMode {
    System,
    User,
    FIQ,
    Supervisor,
    Abort,
    IRQ,
    Undefined
}

impl From<u32> for CpuMode {
    fn from(value: u32) -> Self {
        match value & 0x1f {
            0b10000 => CpuMode::User,
            0b10001 => CpuMode::FIQ,
            0b10010 => CpuMode::IRQ,
            0b10011 => CpuMode::Supervisor,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            0b11111 => CpuMode::System,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct CPU {
    pub registers: [u32; 16],
    // NOTE: General use banked regs, r8-r12
    fiq_banked_gen_regs: [u32; 5],
    // NOTE: Banked regs r13, r14 for all alt modes
    banked_regs: [u32; 4],
    pub cpsr: u32,
    pub spsr: u32,
    psr: [u32; 6],
    pub mode: CpuMode,
    // Should one of these be the addr and the other the value?
    pub execute: u32,
    pub decode: u32,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x03007f00, 0, 0x68],
            fiq_banked_gen_regs: [0; 5],
            banked_regs: [0; 4],
            psr: [0x1f,0,0,0,0,0],
            cpsr: 0x1f,
            mode: CpuMode::System,
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
    pub fn pc(&self) -> usize {
        self.registers[PC] as usize
    }

    // TODO: Do reverse for set_register
    pub fn get_register(&self, rn: usize) -> u32 {
        let mode = CpuMode::from(self.cpsr);
        if rn < 13 && !((mode == CpuMode::FIQ) && rn > 8) {
            self.registers[rn]
        } else if mode == CpuMode::FIQ && rn < 13 {
            self.fiq_banked_gen_regs[rn - 8]
        } else {
            // The 13 and 14 banked regs
            self.banked_regs[rn - 13]
        }
    }

    pub fn set_register(&self, rn: usize, value: u32) {
        todo!()
    }

    pub fn update_cpsr(&mut self, res: u32) {
        self.update_cpsr_with_overflow(res, false);
    }

    pub fn update_cpsr_with_overflow(&mut self, res: u32, overlflow: bool) {
        let zero = if res == 0 {
            CPSR_Z
        } else {
            0
        };

        let neg = if is_signed(res) {
            CPSR_N
        } else {
            0
        };

        // NOTE: When testing 0xFFFFFFFF + 4, it sets C and Z, not V
        let over = if overlflow {
            CPSR_C
        } else {
            0
        };

        self.cpsr &= 0x2fffffff;
        // self.cpsr |= CPSR_C & (res >> 2);
        self.cpsr |= zero;
        self.cpsr |= neg;
        self.cpsr |= over;
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
        let next_inst = if self.is_thumb_mode() {
            ram.read_halfword(self.pc())
        } else {
            ram.read_word(self.pc())
        };

        self.decode = match next_inst {
            Ok(i) => i,
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        // NOTE: I think this has to happen after run
        // that's why the reg is always 8 ahead, and not just 4 ahead
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
            decode_as_thumb(inst)
        };

        op.run(self, ram);
    }
}
