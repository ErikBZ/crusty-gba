use super::cpu::{LR, PC, SP};
use super::{Conditional, Operation, CPSR_T};
use crate::{SystemMemory, CPU};

fn get_triplet_as_usize(value: u32, shift: u32) -> usize {
    (value >> shift & 0x7) as usize
}

fn get_triplet_as_u32(value: u32, shift: u32) -> u32 {
    (value >> shift & 0x7) as u32
}

fn get_triplet_as_u8(value: u32, shift: u32) -> u32 {
    (value >> shift & 0x7) as u32
}

#[derive(Debug, PartialEq)]
struct MoveShiftedRegisterOp {
    op: u8,
    offset: u32,
    rs: usize,
    rd: usize
}

impl From<u32> for MoveShiftedRegisterOp {
    fn from(value: u32) -> Self {
        MoveShiftedRegisterOp {
            op: (value >> 11 & 0x3) as u8,
            offset: value >> 6 & 0x1f,
            rs: (value >> 3 & 0x7) as usize,
            rd: (value & 0x7) as usize,
        }
    }
}

impl Operation for MoveShiftedRegisterOp {
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let res = match self.op {
            0 => cpu.registers[self.rs] << self.offset,
            1 => cpu.registers[self.rs] >> self.offset,
            2 => ((cpu.registers[self.rs] as i32) >> self.offset) as u32,
            _ => unreachable!(),
        };

        cpu.update_cpsr(res);
        cpu.registers[self.rd] = res;
    }
}

#[derive(Debug, PartialEq)]
struct AddSubstractOp {
    i: bool,
    op: bool,
    rn: usize,
    offset: u32,
    rs: usize,
    rd: usize
}

impl From<u32> for AddSubstractOp {
    fn from(value: u32) -> Self {
        AddSubstractOp {
            i: (value >> 10 & 0x1) == 1,
            op: (value >> 9 & 0x1) == 1,
            rn : get_triplet_as_usize(value, 6),
            offset: get_triplet_as_u32(value, 6),
            rs : get_triplet_as_usize(value, 3),
            rd : get_triplet_as_usize(value, 0),       
        }

    }
}

impl Operation for AddSubstractOp {
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let res = match (self.op, self.i) {
            (false, false) => cpu.registers[self.rs] + cpu.registers[self.rn],
            (false, true) => cpu.registers[self.rs] + self.offset,
            (true, false) => cpu.registers[self.rs] - cpu.registers[self.rn],
            (true, true) => cpu.registers[self.rs] - self.offset,
        };

        cpu.update_cpsr(res);
        cpu.registers[self.rd] = res;
    }
}

#[derive(Debug, PartialEq)]
struct MathImmOp {
    op: u8,
    rd: usize,
    offset: u32
}

impl From<u32> for MathImmOp {
    fn from(value: u32) -> Self {
        MathImmOp {
            op: (value >> 11 & 0x3) as u8,
            rd: get_triplet_as_usize(value, 8),
            offset: value & 0xff,
        }
    }
}

impl Operation for MathImmOp {
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let res = match self.op {
            0 => self.offset,
            1 | 3 => cpu.registers[self.rd] - self.offset,
            2 => cpu.registers[self.rd] + self.offset,
            _ => unreachable!(),
        };

        match self.op {
            2 => (),
            _ => cpu.registers[self.rd] = res,
        }
        cpu.update_cpsr(res);
    }
}

#[derive(Debug, PartialEq)]
struct  ALUOp {
    op: u8,
    rs: usize,
    rd: usize,
}

impl From<u32> for ALUOp {
    fn from(value: u32) -> Self {
        ALUOp {
            op: (value >> 6 & 0xf) as u8,
            rs: get_triplet_as_usize(value, 3),
            rd: get_triplet_as_usize(value, 0),
        }
    }
}

impl Operation for ALUOp {
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let res = match self.op {
            0 => cpu.registers[self.rd] & cpu.registers[self.rs],
            1 => cpu.registers[self.rd] ^ cpu.registers[self.rs],
            _ => unreachable!(),
        };

        match self.op {
            9 | 11 | 12 => {},
            _ => cpu.registers[self.rd] = res,
        }
        cpu.update_cpsr(res);
    }
}

#[derive(Debug, PartialEq)]
struct HiRegOp {
    op: u8,
    //Note: if true, adds 8 to rs or rd
    h1: bool,
    h2: bool,
    rs: usize,
    rd: usize,
}

impl From<u32> for HiRegOp {
    fn from(value: u32) -> Self {
        let h1 = (value >> 7 & 1) == 1;
        let h2 = (value >> 6 & 1) == 1;
        let rs = get_triplet_as_usize(value, 3);
        let rd = get_triplet_as_usize(value, 0);
        HiRegOp {
            op: (value >> 8 & 0x3) as u8,
            h1,
            h2,
            rs: if h1 { rs + 8 } else { rs },
            rd: if h2 { rd + 8 } else { rd },
        }
    }
}

impl Operation for HiRegOp {
    fn run(&self, cpu: &mut super::cpu::CPU, mem: &mut SystemMemory) {
        // NOTE: h1 = 0, h2 = 0, op = 00 | 01 | 10 is undefined, and should not be used
        if self.op != 0b11 && !(self.h1 || self.h2) {
            unreachable!();
        }

        match self.op {
            0b00 => cpu.registers[self.rd] += cpu.registers[self.rs],
            0b01 => {
                let res = cpu.registers[self.rd] - cpu.registers[self.rs];
                cpu.update_cpsr(res);
            },
            0b10 => cpu.registers[self.rd] = cpu.registers[self.rs],
            0b11 => {
                let mut addr = cpu.registers[self.rs];
                cpu.update_thumb(addr & 1 == 1);
                addr &= !1;
                // Pipeline flush
                cpu.decode = match mem.read_from_mem(addr as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("Error reading from memory while decoding instruction: {}", e);
                        0
                    }
                };

                if cpu.cpsr & CPSR_T == CPSR_T {
                    cpu.registers[PC] = addr + 2;
                } else {
                    cpu.registers[PC] = addr + 4;
                }
            },
            _ => unreachable!(), 
        }
    }
}

#[derive(Debug, PartialEq)]
struct PcRelativeLoadOp {
    rd: usize,
    word: u32,
}

impl From<u32> for PcRelativeLoadOp {
    fn from(value: u32) -> Self {
        PcRelativeLoadOp {
            rd: get_triplet_as_usize(value, 8),
            word: (value & 0xff) as u32,
        }
    }
}

impl Operation for PcRelativeLoadOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        // NOTE: The value of PC will always be 4 bytes greater, but bit 1 of PC will always be 0
        let offset = self.word << 2; 
        let addr = (cpu.registers[PC] + offset) as usize;

        let block_from_mem = match mem.read_from_mem(addr) {
            Ok(n) => n,
            Err(e) => {
                println!("{}", e);
                panic!()
            },
        };

        cpu.registers[self.rd] = block_from_mem;
    }
}

#[derive(Debug, PartialEq)]
struct LoadStoreRegOffsetOp {
    l: bool,
    b: bool,
    ro: usize,
    rb: usize,
    rd: usize,
}

impl From<u32> for LoadStoreRegOffsetOp {
    fn from(value: u32) -> Self {
        LoadStoreRegOffsetOp {
            l: (value >> 11 & 1) == 1,
            b: (value >> 10 & 1) == 1,
            ro: get_triplet_as_usize(value, 6),
            rb: get_triplet_as_usize(value, 3),
            rd: get_triplet_as_usize(value, 0),
        }
    }
}

impl Operation for LoadStoreRegOffsetOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.registers[self.rb] + cpu.registers[self.ro]) as usize;
        if self.l {
            let block_from_mem = match mem.read_from_mem(addr) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    panic!()
                },
            };

            cpu.registers[self.rd] = if self.b {
                block_from_mem
            } else {
                block_from_mem & 0xff
            };
        } else {
            if self.b {
                match mem.write_word(addr, cpu.registers[self.rd]) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            } else {
                match mem.write_byte(addr, cpu.registers[self.rd]) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct LoadStoreSignExOp {
    h: bool,
    s: bool,
    ro: usize,
    rb: usize,
    rd: usize,
}

impl From<u32> for LoadStoreSignExOp {
    fn from(value: u32) -> Self {
        LoadStoreSignExOp {
            h: (value >> 11 & 1) == 1,
            s: (value >> 10 & 1) == 1,
            ro: get_triplet_as_usize(value, 6),
            rb: get_triplet_as_usize(value, 3),
            rd: get_triplet_as_usize(value, 0),
        }
    }
}

impl Operation for LoadStoreSignExOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.registers[self.rb] + cpu.registers[self.ro]) as usize;
        if !self.h && !self.s {
            match mem.write_word(addr, cpu.registers[self.rd]) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e)
                }
            }
        } else {
            let block_from_mem = match mem.read_from_mem(addr) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    panic!()
                },
            };

            cpu.registers[self.rd] = block_from_mem;
        }
    }
}

#[derive(Debug, PartialEq)]
struct LoadStoreImmOffsetOp {
    b: bool,
    l: bool,
    offset: u32,
    rb: usize,
    rd: usize,
}

impl From<u32> for LoadStoreImmOffsetOp {
    fn from(value: u32) -> Self {
        LoadStoreImmOffsetOp {
            b: (value >> 12 & 1) == 1,
            l: (value >> 11 & 1) == 1,
            offset: value >> 6 & 0x1f,
            rb: get_triplet_as_usize(value, 3),
            rd: get_triplet_as_usize(value, 0),
        }
    }
}

impl Operation for LoadStoreImmOffsetOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.registers[self.rb] + if self.b { self.offset } else { self.offset << 2 }) as usize;
        if self.l {
            let res = if self.b {
                match mem.read_byte(addr) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                }
            } else {
                match mem.read_word(addr) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                }
            };

            cpu.registers[self.rd] = res;
        } else {
            let res = if self.b {
                mem.write_byte(addr, cpu.registers[self.rd])
            } else {
                mem.write_word(addr, cpu.registers[self.rd])
            };

            match res {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct LoadStoreHalfWordOp {
    l: bool,
    offset: u32,
    rb: usize,
    rd: usize,
}

impl From<u32> for LoadStoreHalfWordOp {
    fn from(value: u32) -> Self {
        LoadStoreHalfWordOp {
            l: (value >> 11 & 1) == 1,
            offset: (value >> 5 & 0x3e) as u32,
            rb: get_triplet_as_usize(value, 3),
            rd: get_triplet_as_usize(value, 0),
        }
    }
}

impl Operation for LoadStoreHalfWordOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.registers[self.rb] + self.offset << 1) as usize;
        if self.l {
            match mem.write_halfword(addr, cpu.registers[self.rd]) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e)
                }
            }
        } else {
            let block_from_mem = match mem.read_halfword(addr) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    panic!()
                },
            };

            cpu.registers[self.rd] = block_from_mem;
        }
    }
}

#[derive(Debug, PartialEq)]
struct SpRelativeLoadOp {
    l: bool,
    rd: usize,
    word: u32
}

impl From<u32> for SpRelativeLoadOp {
    fn from(value: u32) -> Self {
        SpRelativeLoadOp {
            l: (value >> 11 & 1) == 1,
            rd: get_triplet_as_usize(value, 8),
            word: (value & 0xff) << 2,
        }
    }
}

impl Operation for SpRelativeLoadOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.registers[7] + self.word) as usize;

        if self.l {
            match mem.write_word(addr, cpu.registers[self.rd]) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                    panic!();
                }
            }
        } else {
            let block_from_mem = match mem.read_word(addr) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    panic!();
                },
            };

            cpu.registers[self.rd] = block_from_mem;
        }
    }
}

#[derive(Debug, PartialEq)]
struct LoadAddressOp {
    sp: bool,
    rd: usize,
    word: u32
}

impl From<u32> for LoadAddressOp {
    fn from(value: u32) -> Self {
        LoadAddressOp {
            sp: (value >> 11 & 0x1) == 1,
            rd: get_triplet_as_usize(value, 8),
            word: (value & 0xff) << 2, 
        }
    }
}

impl Operation for LoadAddressOp {
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        let res = if !self.sp {
            (cpu.registers[PC] & !1) + self.word + 4
        } else {
            cpu.registers[SP] + self.word
        };

        cpu.registers[self.rd] = res;
    }
}

#[derive(Debug, PartialEq)]
struct AddOffsetSPOp {
    s: bool,
    word: u32,
}

impl From<u32> for AddOffsetSPOp {
    fn from(value: u32) -> Self {
        AddOffsetSPOp {
            s: (value >> 7 & 1) == 1,
            word: value & 0x7f << 2,
        }
    }
}

impl Operation for AddOffsetSPOp {
    // TODO: This may need to be updated
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        if self.s {
            cpu.registers[SP] -= self.word;
        } else {
            cpu.registers[SP] += self.word;
        }
    }
}

#[derive(Debug, PartialEq)]
struct PushPopRegOp {
    l: bool,
    r: bool,
    rlist: u8,
}

impl From<u32> for PushPopRegOp {
    fn from(value: u32) -> Self {
        PushPopRegOp {
            l: (value >> 11 & 1) == 1,
            r: (value >> 8 & 1) == 1,
            rlist: (value & 0xff) as u8,
        }
    }
}

impl Operation for PushPopRegOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        for i in 0..8 {
            if (self.rlist >> i & 1) == 0 {
                continue;
            }

            if self.l {
                cpu.registers[i] = match mem.read_word(cpu.registers[SP] as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                };
                cpu.registers[SP] -= 4;
            } else {
                match mem.write_word(cpu.registers[SP] as usize, cpu.registers[i]) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                };
                cpu.registers[SP] += 4;
            }
        }

        // TODO: Find more consise way of checking the 'r' register LR or PC
        if self.r {
            if self.l {
                // If updating PC, should we have to flush the pipline?
                cpu.registers[PC] = match mem.read_word(cpu.registers[SP] as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                };
                cpu.registers[SP] -= 4;
            } else {
                match mem.write_word(cpu.registers[SP] as usize, cpu.registers[LR]) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                };
                cpu.registers[SP] += 4;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct MultipleLoadStoreOp {
    l: bool,
    rb: usize,
    rlist: u8,
}

impl From<u32> for MultipleLoadStoreOp {
    fn from(value: u32) -> Self {
        MultipleLoadStoreOp {
            l: (value >> 11 & 1) == 1,
            rb: get_triplet_as_usize(value, 8),
            rlist: (value & 0xff) as u8,
        }
    }
}

impl Operation for MultipleLoadStoreOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        for i in 0..8 {
            if (self.rlist >> i & 1) == 0 {
                continue;
            }

            if self.l {
                cpu.registers[i] = match mem.read_word(cpu.registers[self.rb] as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                };
                cpu.registers[self.rb] -= 4;
            } else {
                match mem.write_word(cpu.registers[self.rb] as usize, cpu.registers[i]) {
                    Ok(_) => (),
                    Err(e) => println!("{}", e),
                };
                cpu.registers[self.rb] += 4;
            }
        }
    }
}

#[derive(Debug, PartialEq)]
struct ConditionalBranchOp {
    cond: Conditional,
    offset: u32,
}

impl From<u32> for ConditionalBranchOp {
    fn from(value: u32) -> Self {
        ConditionalBranchOp {
            offset: (value & 0xff) as u32,
            cond: match value >> 8 & 0xff {
                0  => Conditional::EQ,
                1  => Conditional::NE,
                2  => Conditional::CS,
                3  => Conditional::CC,
                4  => Conditional::MI,
                5  => Conditional::PL,
                6  => Conditional::VS,
                7  => Conditional::VC,
                8  => Conditional::HI,
                9  => Conditional::LS,
                10 => Conditional::GE,
                11 => Conditional::LT,
                12 => Conditional::GT,
                13 => Conditional::LE,
                _ => unreachable!()
            },
        }
    }
}

impl Operation for ConditionalBranchOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        if !self.cond.should_run(cpu.cpsr) {
            return;
        }

        let offset = if self.offset & (1 << 8) == (1 << 8) {
            ((self.offset << 1) | 0xfffffe00) as i32
        } else {
            (self.offset << 1) as i32
        };

        let offset_abs: u32 = u32::try_from(offset.abs()).unwrap_or(0);

        let addr = if offset < 0 {
            cpu.registers[PC] - offset_abs
        } else {
            cpu.registers[PC] + offset_abs
        };

        cpu.decode = match mem.read_from_mem(addr as usize) {
            Ok(n) => n,
            Err(_) => 0,
        };

        cpu.registers[PC] = addr + 4;
    }
}

#[derive(Debug, PartialEq)]
struct SoftwareInterruptOp {
    value: u32,
}

impl From<u32> for SoftwareInterruptOp {
    fn from(value: u32) -> Self {
        SoftwareInterruptOp {
            value: (value & 0xff) as u32,
        }
    }
}

impl Operation for SoftwareInterruptOp {
    fn run(&self, _cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        // Move address of next instruction into LR, Copy CPSR to SPSR
        // Load SWI Vector Address into PC, swith to ARM mode, enter SVC
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct UnconditionalBranchOp {
    offset: u32,
}

impl From<u32> for UnconditionalBranchOp {
    fn from(value: u32) -> Self {
        UnconditionalBranchOp {
            offset: (value & 0x7ff) << 1,
        } 
    } 
}

impl Operation for UnconditionalBranchOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let offset = if self.offset & (1 << 11) == (1 << 11) {
            ((self.offset << 1) | 0xfffff800) as i32
        } else {
            (self.offset << 1) as i32
        };

        let offset_abs: u32 = u32::try_from(offset.abs()).unwrap_or(0);

        let addr = if offset < 0 {
            cpu.registers[PC] - offset_abs
        } else {
            cpu.registers[PC] + offset_abs
        };

        cpu.decode = match mem.read_from_mem(addr as usize) {
            Ok(n) => n,
            Err(_) => 0,
        };

        cpu.registers[PC] = addr + 2;
    }
}

#[derive(Debug, PartialEq)]
struct LongBranchWithLinkOp {
    h: bool,
    offset: u32,
}

impl From<u32> for LongBranchWithLinkOp {
    fn from(value: u32) -> Self {
        LongBranchWithLinkOp {
            h: (value >> 11 & 1) == 1,
            offset: value & 0x7ff,
        }
    } 
}

impl Operation for LongBranchWithLinkOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        // !self.h runs first, the next addr MUST be another LongBranchWithLinkOp
        // with self.h == true
        if !self.h {
            cpu.registers[LR] = cpu.registers[PC] + self.offset << 12;
        } else {
            let temp = cpu.registers[PC] - 2;
            cpu.registers[PC] = cpu.registers[LR] + self.offset << 1;
            cpu.registers[LR] = temp;

            cpu.decode = match mem.read_halfword(cpu.registers[PC] as usize) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    0
                }
            };

            cpu.registers[PC] +=2;
        }
    }
}

pub fn decode_as_thumb(value: u32) -> Box<dyn Operation> {
    if value & 0xf800 == 0x1800 {
        // AddSubstractOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xe000 == 0x0 {
        // MoveShiftedRegisterOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xe000 == 0x2000 {
        // MathImmOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xfc00 == 0x2000 {
        // ALUOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xfc00 == 0x4400 {
        // HiRegOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf800 == 0x4800 {
        // PcRelativeLoadOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf200 == 0x5000 {
        // LoadStoreRegOffsetOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf200 == 0x5200 {
        // LoadStoreSignExOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xe000 == 0x6000 {
        // LoadStoreHalfWordOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0x8000 {
        // LoadStoreImmOffsetOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0x9000 {
        // SpRelativeLoadOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0xa000 {
        // LoadAddressOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xff00 == 0xb000 {
        // AddOffsetSPOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf600 == 0xb400 {
        // PushPopRegOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0xc000 {
        // MultipleLoadStoreOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0xd000 {
        // ConditionalBranchOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf800 == 0xe000 {
        // UnconditionalBranchOp
        Box::new(AddSubstractOp::from(value))
    } else if value & 0xf000 == 0xf000 {
        // LongBranchWithLinkOp
        Box::new(AddSubstractOp::from(value))
    } else {
        Box::new(AddSubstractOp::from(value))
    }
}

mod test {
    use super::*;

    #[test]
    fn test_lsl_decode() {
        let inst: u32 = 0x0636;
        let op = MoveShiftedRegisterOp::from(inst);
        assert_eq!(op, MoveShiftedRegisterOp{op:0 ,rd: 6, rs: 6, offset: 0x18});
    }

    #[test]
    fn test_add_reg_variant() {
        let inst: u32 = 0x19ad;
        let op = AddSubstractOp::from(inst);
        assert_eq!(op, AddSubstractOp{rd: 5, rs: 5, rn: 6, i: false, op: false, offset: 6});
    }

    #[test]
    fn test_sub_imm_variant() {
        let inst: u32 = 0x1e68;
        let op = AddSubstractOp::from(inst);
        assert_eq!(op, AddSubstractOp{rd: 0, rs: 5, rn: 1, i: true, op: true, offset: 1});
    }

    #[test]
    fn test_add_imm_variant() {
        let inst: u32 = 0x1c22;
        let op = AddSubstractOp::from(inst);
        assert_eq!(op, AddSubstractOp{rd: 2, rs: 4, offset: 0, rn: 0, i: true, op: false});
    }

    #[test]
    fn test_mov_imm_variant() {
        let inst: u32 = 0x2400;
        let op = MathImmOp::from(inst);
        assert_eq!(op, MathImmOp{rd: 4, offset: 0, op: 0});
    }

    #[test]
    fn test_add_byte_imm_vairant() {
        let inst: u32 = 0x3210;
        let op = MathImmOp::from(inst);
        assert_eq!(op, MathImmOp{rd: 2, offset: 0x10, op: 2});
    }

    #[test]
    fn test_bx_variant_one() {
        let inst: u32 = 0x4770;
        let op = HiRegOp::from(inst);
        assert_eq!(op, HiRegOp{h1: false, h2: true, rd: 0, rs: 6, op: 3});
    }

    #[test]
    fn test_bx_variant_two() {
        let inst: u32 = 0x4718;
        let op = HiRegOp::from(inst);
        assert_eq!(op, HiRegOp{h1: false, h2: false, rd: 0, rs: 3, op: 3});
    }

    #[test]
    fn test_ldr_decode() {
        let inst: u32 = 0x49f8;
        let op = PcRelativeLoadOp::from(inst);
        // TODO: in ghidra this is DAT_0000ac0
        assert_eq!(op, PcRelativeLoadOp{rd: 1, word: 0xf8});
    }

    #[test]
    fn test_ldrb_decode() {
        let inst: u32 = 0x5d82;
        let op = LoadStoreRegOffsetOp::from(inst);
        assert_eq!(op, LoadStoreRegOffsetOp{l: true, b: true, rd: 2, rb: 0, ro: 6});
    }

    #[test]
    fn test_strh_decode() {
        let inst: u32 = 0x81bb;
        let op = LoadStoreHalfWordOp::from(inst);
        assert_eq!(op, LoadStoreHalfWordOp{l: false, offset: 0xc, rb: 7, rd: 3});
    }

    #[test]
    fn test_b_decode() {
        let inst: u32 = 0xe3a0;
        let op = UnconditionalBranchOp::from(inst);
        assert_eq!(op, UnconditionalBranchOp{offset: 0x740});
    }

    #[test]
    fn test_push_decode() {
        let inst: u32 = 0xb578;
        let op = PushPopRegOp::from(inst);
        assert_eq!(op, PushPopRegOp{l: false, r: true, rlist: 0b1111000});
    }

    #[test]
    fn test_strh_decode_two() {
        let inst: u32 = 0x7090;
        let op = LoadStoreImmOffsetOp::from(inst);
        assert_eq!(op, LoadStoreImmOffsetOp{offset: 2, rd: 0, rb: 2, l: false, b: true});
    }
}
