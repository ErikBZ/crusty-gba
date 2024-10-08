pub mod cpu;
pub mod debugger;
pub mod arm;
pub mod thumb;
pub mod system;

pub const CPSR_N: u32 = 0x80000000;
pub const CPSR_Z: u32 = 0x60000000;
pub const CPSR_C: u32 = 0x20000000;
pub const CPSR_V: u32 = 0x10000000;
pub const CPSR_T: u32 = 0x20;

use crate::SystemMemory;

// Operations can be ARM or Thumb instructions
pub trait Operation: std::fmt::Debug {
    fn run(&self, cpu: &mut cpu::CPU, mem: &mut SystemMemory);
}

#[derive(Debug, strum_macros::Display, PartialEq)]
pub enum Conditional {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    #[strum(to_string = "")]
    AL,
    NV,
}

impl From<u32> for Conditional {
    fn from(instruction: u32) -> Self {
        let conditional = instruction >> 28;
        match conditional {
            0 => Conditional::EQ,
            1 => Conditional::NE,
            2 => Conditional::CS,
            3 => Conditional::CC,
            4 => Conditional::MI,
            5 => Conditional::PL,
            6 => Conditional::VS,
            7 => Conditional::VC,
            8 => Conditional::HI,
            9 => Conditional::LS,
            10 => Conditional::GE,
            11 => Conditional::LT,
            12 => Conditional::GT,
            13 => Conditional::LE,
            14 => Conditional::AL,
            _ => Conditional::NV,
        }
    }
}

impl Conditional {
    pub fn should_run(&self, cpsr: u32) -> bool {
        match self {
            Conditional::EQ => {
                (cpsr & CPSR_Z) == CPSR_Z
            },
            Conditional::NE => {
                (cpsr & CPSR_Z) == 0
            },
            Conditional::CS => {
                (cpsr & CPSR_C) == CPSR_C
            },
            Conditional::CC => {
                (cpsr & CPSR_C) == 0
            },
            Conditional::MI => {
                (cpsr & CPSR_N) == CPSR_N
            },
            Conditional::PL => {
                (cpsr & CPSR_N) == 0
            },
            Conditional::VS => {
                (cpsr & CPSR_V) == CPSR_V
            },
            Conditional::VC => {
                (cpsr & CPSR_V) == 0
            },
            Conditional::HI => {
                (cpsr & CPSR_C) == CPSR_C && (cpsr & CPSR_Z) == 0
            },
            Conditional::LS => {
                (cpsr & CPSR_C) == 0 && (cpsr & CPSR_Z) == CPSR_Z
            },
            Conditional::GE => {
                (cpsr & CPSR_N) == (cpsr & CPSR_V << 3)
            },
            Conditional::LT => {
                (cpsr & CPSR_N) != (cpsr & CPSR_V << 3)
            },
            Conditional::GT => {
                (cpsr & CPSR_Z) == 0 && (cpsr & CPSR_N == cpsr & CPSR_V << 3)
            },
            Conditional::LE => {
                (cpsr & CPSR_Z) == CPSR_Z || (cpsr & CPSR_N != cpsr & CPSR_V << 3)
            },
            Conditional::AL => {
                true
            },
            _ => false,
        }
    }
}

