use super::cpu::{LR, PC, SP};
use super::system::{read_cycles_per_32, read_cycles_per_8_16};
use super::{add_nums, bit_map_to_array, count_cycles, get_abs_int_value, get_v_from_add, get_v_from_sub, is_signed, subtract_nums, Conditional, Operation, CPSR_C, CPSR_T};
use crate::utils::shifter::ShiftWithCarry;
use crate::{SystemMemory, CPU};
use crate::gba::Memory;
use tracing::warn;

pub fn decode_as_thumb(value: u32) -> Box<dyn Operation> {
    if value & 0xf800 == 0x1800 {
        // AddSubtractOp
        Box::new(AddSubtractOp::from(value))
    } else if value & 0xe000 == 0x0 {
        // MoveShiftedRegisterOp
        Box::new(MoveShiftedRegisterOp::from(value))
    } else if value & 0xe000 == 0x2000 {
        // MathImmOp
        Box::new(MathImmOp::from(value))
    } else if value & 0xfc00 == 0x4000 {
        // ALUOp
        Box::new(ALUOp::from(value))
    } else if value & 0xfc00 == 0x4400 {
        // HiRegOp
        Box::new(HiRegOp::from(value))
    } else if value & 0xf800 == 0x4800 {
        // PcRelativeLoadOp
        Box::new(PcRelativeLoadOp::from(value))
    } else if value & 0xf200 == 0x5000 {
        // LoadStoreRegOffsetOp
        Box::new(LoadStoreRegOffsetOp::from(value))
    } else if value & 0xf200 == 0x5200 {
        // LoadStoreSignExOp
        Box::new(LoadStoreSignExOp::from(value))
    } else if value & 0xe000 == 0x6000 {
        // LoadStoreImmOffsetOp
        Box::new(LoadStoreImmOffsetOp::from(value))
    } else if value & 0xf000 == 0x8000 {
        // LoadStoreHalfWordOp
        Box::new(LoadStoreHalfWordOp::from(value))
    } else if value & 0xf000 == 0x9000 {
        // SpRelativeLoadOp
        Box::new(SpRelativeLoadOp::from(value))
    } else if value & 0xf000 == 0xa000 {
        // LoadAddressOp
        Box::new(LoadAddressOp::from(value))
    } else if value & 0xff00 == 0xb000 {
        // AddOffsetSPOp
        Box::new(AddOffsetSPOp::from(value))
    } else if value & 0xf600 == 0xb400 {
        // PushPopRegOp
        Box::new(PushPopRegOp::from(value))
    } else if value & 0xf000 == 0xc000 {
        // MultipleLoadStoreOp
        Box::new(MultipleLoadStoreOp::from(value))
    } else if value & 0xf000 == 0xd000 {
        // ConditionalBranchOp
        Box::new(ConditionalBranchOp::from(value))
    } else if value & 0xf800 == 0xe000 {
        // UnconditionalBranchOp
        Box::new(UnconditionalBranchOp::from(value))
    } else if value & 0xf000 == 0xf000 {
        // LongBranchWithLinkOp
        Box::new(LongBranchWithLinkOp::from(value))
    } else {
        Box::new(SoftwareInterruptOp::from(value))
    }
}

fn get_triplet_as_usize(value: u32, shift: u32) -> usize {
    (value >> shift & 0x7) as usize
}

fn get_triplet_as_u32(value: u32, shift: u32) -> u32 {
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

// TODO: Set carry for these
impl Operation for MoveShiftedRegisterOp {
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let rs = cpu.get_register(self.rs);
        let (res, c_carry) = match self.op {
            0 => rs.shl_with_carry(self.offset),
            1 => rs.shr_with_carry(self.offset),
            2 => rs.asr_with_carry(self.offset),
            _ => unreachable!(),
        };

        // TODO: This probably sets carry
        cpu.update_cpsr(res, false, c_carry);
        cpu.set_register(self.rd, res);
        let mut cycles = 1;
        if self.rd == PC {
            // NOTE: 1S + 1N
            cycles += 2;
        }
        // NOTE: 1S + 1I (for shift)
        cpu.add_cycles(cycles);
    }
}

#[derive(Debug, PartialEq)]
struct AddSubtractOp {
    i: bool,
    op: bool,
    rn: usize,
    offset: u32,
    rs: usize,
    rd: usize
}

impl From<u32> for AddSubtractOp {
    fn from(value: u32) -> Self {
        AddSubtractOp {
            i: (value >> 10 & 0x1) == 1,
            op: (value >> 9 & 0x1) == 1,
            rn : get_triplet_as_usize(value, 6),
            offset: get_triplet_as_u32(value, 6),
            rs : get_triplet_as_usize(value, 3),
            rd : get_triplet_as_usize(value, 0),       
        }

    }
}

impl Operation for AddSubtractOp {
    // TODO: PC is being tracked incorrectly. Gotta fix that
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let offset = if self.i {
            self.offset
        } else {
            cpu.get_register(self.rn)
        };

        let (res, v_status) = if self.op {
            subtract_nums(cpu.get_register(self.rs), offset, false)
        } else {
            add_nums(cpu.get_register(self.rs), offset, false)
        };

        let c_status = res >> 32 & 1 == 1;
        let res = 0xffffffff & res as u32;

        cpu.update_cpsr(res, v_status, c_status);
        cpu.set_register(self.rd, res);

        if self.rd == PC {
            // NOTE: 1S + 1N
            cpu.add_cycles(3);
        } else {
            // NOTE: 1S
            cpu.add_cycles(1);
        }
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
    // TODO: Improve this, looks too ugly
    fn run(&self, cpu: &mut super::cpu::CPU, _mem: &mut SystemMemory) {
        let rd = cpu.get_register(self.rd) as u64;
        let mut v_status = false;

        let res = match self.op {
            0 => {
                let res = self.offset as u64;
                if cpu.cpsr & CPSR_C != 0 {
                    res | (1 << 32)
                } else {
                    res
                }
            },
            1 | 3 => {
                let (x, v_stat) = subtract_nums(cpu.get_register(self.rd), self.offset, false);
                v_status = v_stat;
                x
            },
            2 => {
                let offset = self.offset as u64;
                let res = rd + offset;
                v_status = get_v_from_add(rd, offset, res);
                res
            }
            _ => unreachable!(),
        };

        let c_status = (res >> 32) & 1 == 1;
        let res = (res & 0xffffffff) as u32;

        match self.op {
            1 => (),
            _ => cpu.set_register(self.rd, res),
        }
        cpu.update_cpsr(res, v_status, c_status);
        if self.rd == PC {
            // NOTE: 2S + 1N + 1I
            cpu.add_cycles(3)
        } else {
            // NOTE: 1S + 1I
            cpu.add_cycles(1)
        }
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
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        let rd_value = cpu.get_register(self.rd) as u64;
        let rs_value = cpu.get_register(self.rs) as u64;
        let carry = ((cpu.cpsr & CPSR_C) >> 29) as u64;
        let mut v_status = false;

        // TODO: Maybe make self.op enum?
        let res = match self.op {
            // TODO: Implement Shift check for carry
            0 => rd_value & rs_value,
            1 => rd_value ^ rs_value,
            2 => rd_value >> rs_value,
            3 => rd_value << rs_value,
            4 => ((cpu.get_register(self.rd) as i32) >> rs_value) as u64,
            5 => {
                let res = rd_value + rs_value + carry;
                v_status = get_v_from_add(rd_value, rs_value, res);
                res
            },
            6 => {
                let rhs = !cpu.get_register(self.rs) as u64;
                let res = rd_value + rhs + carry;
                v_status = get_v_from_sub(rd_value, rs_value, res);
                res
            },
            7 => cpu.get_register(self.rd).rotate_right(cpu.get_register(self.rs)) as u64,
            8 => rd_value & rs_value,
            9 => {
                let res = !cpu.get_register(self.rd) as u64;
                res + 1
            },
            10 => {
                let rhs = !cpu.get_register(self.rs) as u64;
                let res = rd_value + rhs + 1;
                v_status = get_v_from_sub(rd_value, rs_value, res);
                res
            },
            11 => {
                let res = rd_value + rs_value;
                v_status = get_v_from_add(rd_value, rs_value, res);
                res
            },
            12 => rd_value | rs_value,
            // TODO: This is gonna cause issues
            13 => {
                rd_value.wrapping_mul(rs_value)
            },
            14 => rd_value & !rs_value,
            15 => !rs_value,
            _ => unreachable!(),
        };

        let c_status = (res >> 32) & 1 == 1;
        let res = (res & 0xffffffff) as u32;

        match self.op {
            9 | 10 | 11 => {},
            _ => cpu.set_register(self.rd, res),
        }

        let cycles = match self.op {
            7 => 2,
            13 => count_cycles(rs_value as u32),
            _ => 1,
        };

        cpu.update_cpsr(res, v_status, c_status);
        cpu.add_cycles(cycles);
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
            rd: if h1 { rd + 8 } else { rd },
            rs: if h2 { rs + 8 } else { rs },
        }
    }
}

impl Operation for HiRegOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let rd = cpu.get_register(self.rd);
        let rs = cpu.get_register(self.rs);
        // NOTE: h1 = 0, h2 = 0, op = 00 | 01 | 10 is undefined, and should not be used
        if self.op != 0b11 && !(self.h1 || self.h2) {
            unreachable!();
        }

        match self.op {
            0b00 => cpu.set_register(self.rd, rd + rs),
            0b01 => {
                let (res, v_status) = subtract_nums(rd, rs, false);
                let c_status = (res >> 32) & 1 == 1;
                cpu.update_cpsr((res & 0xffffffff) as u32, v_status, c_status);
            },
            0b10 => cpu.set_register(self.rd, rs),
            0b11 => {
                let mut addr = cpu.get_register(self.rs);
                cpu.update_thumb(addr & 1 == 1);
                addr &= !1;

                let next_inst = if cpu.is_thumb_mode() {
                    mem.read_halfword(addr as usize)
                } else {
                    mem.read_word(addr as usize)
                };

                // Pipeline flush
                cpu.decode = match next_inst {
                    Ok(n) => n,
                    Err(e) => {
                        warn!("Error reading from memory while decoding instruction: {}", e);
                        0
                    }
                };
                cpu.inst_addr = addr as usize;

                if cpu.cpsr & CPSR_T == CPSR_T {
                    cpu.set_register(PC, addr + 2);
                } else {
                    cpu.set_register(PC, addr + 4);
                }
            },
            _ => unreachable!(), 
        }

        if self.op == 0b11 {
            cpu.add_cycles(3);
        } else {
            cpu.add_cycles(1);
        }
    }
}

fn cycles_for_str_ldr(l: bool, pc: bool, cycles: u32) -> u32 {
    if l && pc {
        // 2S + 2N + 1I
        4 + cycles
    } else if l && !pc {
        // 1S + 1N + 1I
        2 + cycles
    } else {
        // 2N
        2
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
        let addr = (cpu.get_register(PC) + offset) as usize;

        let block_from_mem = match mem.read_word(addr) {
            Ok(n) => n,
            Err(e) => {
                warn!("{}", e);
                panic!()
            },
        };

        cpu.set_register(self.rd, block_from_mem);
        cpu.add_cycles(
            // TOOD: will this ever be anything other than 1?
            cycles_for_str_ldr(true, self.rd == PC, 1)
        )
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
        let ro_val = cpu.get_register(self.ro);
        let offset = get_abs_int_value(ro_val);

        let addr = if is_signed(ro_val) {
            (cpu.get_register(self.rb) - offset) as usize
        } else {
            (cpu.get_register(self.rb) + offset) as usize
        };

        if self.l {
            let block = if self.b {
                mem.read_byte(addr)
            } else {
                mem.read_word(addr)
            };

            let data = match block {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    panic!()
                }
            };
            cpu.set_register(self.rd, data);
        } else {
            // TODO: Rewrite with let x if, and match on the result x
            let res = if self.b {
                mem.write_word(addr, cpu.get_register(self.rd))
            } else {
                mem.write_byte(addr, cpu.get_register(self.rd))
            };

            match res {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
        }

        let cycles = if self.b {
            read_cycles_per_8_16(addr)
        } else {
            read_cycles_per_32(addr)
        };

        cpu.add_cycles(
            cycles_for_str_ldr(self.l, self.rd == PC, cycles)
        );
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
        let addr = (cpu.get_register(self.rb) + cpu.get_register(self.ro)) as usize;
        if !self.h && !self.s {
            match mem.write_halfword(addr, cpu.get_register(self.rd)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
        } else {
            let data = if self.h && !self.s{
                mem.read_halfword(addr)
            } else if !self.h && self.s {
                mem.read_byte_sign_ex(addr)
            } else {
                mem.read_halfword_sign_ex(addr)
            };

            let data = match data {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    0
                },
            };

            cpu.set_register(self.rd, data);
        }

        let cycles = read_cycles_per_8_16(addr);
        cpu.add_cycles(
            cycles_for_str_ldr(self.s || self.h, self.rd == PC, cycles)
        );
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
        let addr = (cpu.get_register(self.rb) + if self.b { self.offset } else { self.offset << 2 }) as usize;
        if self.l {
            let val = if self.b {
                mem.read_byte(addr)
            } else {
                mem.read_word(addr)
            };

            let res = match val {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    0
                }
            };
            cpu.set_register(self.rd, res);
        } else {
            let res = if self.b {
                mem.write_byte(addr, cpu.get_register(self.rd))
            } else {
                mem.write_word(addr, cpu.get_register(self.rd))
            };

            match res {
                Ok(_) => (),
                Err(e) => warn!("{}", e),
            }
        }

        let cycles = if self.b {
            read_cycles_per_8_16(addr)
        } else {
            read_cycles_per_32(addr)
        };

        cpu.add_cycles(
            cycles_for_str_ldr(self.l, self.rd == PC, cycles)
        );
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
        let addr = (cpu.get_register(self.rb) + self.offset) as usize;
        if self.l {
            let block_from_mem = match mem.read_halfword(addr) {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    panic!()
                },
            };

            cpu.set_register(self.rd, block_from_mem);
        } else {
            match mem.write_halfword(addr, cpu.get_register(self.rd)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
        }

        let cycles = read_cycles_per_32(addr);

        cpu.add_cycles(
            cycles_for_str_ldr(self.l, self.rd == PC, cycles)
        );
    }
}

#[derive(Debug, PartialEq)]
struct SpRelativeLoadOp {
    l: bool,
    rd: usize,
    offset: u32
}

impl From<u32> for SpRelativeLoadOp {
    fn from(value: u32) -> Self {
        SpRelativeLoadOp {
            l: (value >> 11 & 1) == 1,
            rd: get_triplet_as_usize(value, 8),
            offset: (value & 0xff) << 2,
        }
    }
}

impl Operation for SpRelativeLoadOp {
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let addr = (cpu.get_register(SP) + self.offset) as usize;

        if self.l {
            let block_from_mem = match mem.read_word(addr) {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    panic!();
                },
            };

            cpu.set_register(self.rd, block_from_mem);
        } else {
            match mem.write_word(addr, cpu.get_register(self.rd)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e);
                    panic!();
                }
            }
        }
        let cycles = read_cycles_per_32(addr);

        cpu.add_cycles(
            cycles_for_str_ldr(self.l, self.rd == PC, cycles)
        );
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
        let res = if self.sp {
            cpu.get_register(SP) + self.word
        } else {
            // TODO: Adding 2 here because it's 2 ahead where it should be.
            // BUG: Fix Pipeline
            (cpu.get_register(PC) & !3) + self.word
        };

        cpu.set_register(self.rd, res);
        if self.rd == PC {
            // NOTE: (ALU with PC) 2S + 1N
            cpu.add_cycles(3)
        } else {
            // NOTE: (ALU) 1S
            cpu.add_cycles(1)
        }
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
            word: (value & 0x7f) << 2,
        }
    }
}

impl Operation for AddOffsetSPOp {
    // TODO: This may need to be updated
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        if self.s {
            cpu.set_register(SP, cpu.get_register(SP) - self.word);
        } else {
            cpu.set_register(SP, cpu.get_register(SP) + self.word);
        }
        // NOTE: (ALU) 1S
        cpu.add_cycles(1);
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
        let mut registers = bit_map_to_array(self.rlist as u32);
        if self.r {
            registers.push(if self.l { PC as u32 } else { LR as u32 });
        }

        for i in 0..registers.len() {
            if self.l {
                let reg = registers[i];
                let value = match mem.read_word(cpu.get_register(SP) as usize) {
                    Ok(n) => {
                        if reg == PC as u32 {
                            n & 0xfffffffe
                        } else {
                            n
                        }
                    },
                    Err(e) => {
                        warn!("{}", e);
                        0
                    }
                };
                cpu.set_register(reg as usize, value);
                // TODO: Super Hacky, update pipeline. This should need to be done and, we should fetch inst at the
                // end i think.
                if reg == PC as u32 {
                    let addr = cpu.get_register(PC) as usize;
                    let next_inst = if cpu.is_thumb_mode() {
                        mem.read_halfword(addr as usize)
                    } else {
                        mem.read_word(addr as usize)
                    };

                    cpu.decode = match next_inst {
                        Ok(n) => n,
                        Err(_) => panic!(),
                    };
                    cpu.set_register(reg as usize, (addr + 2) as u32)
                }

                cpu.set_register(SP, cpu.get_register(SP) + 4);
            } else {
                let reg = registers[registers.len() - i - 1];
                cpu.set_register(SP, cpu.get_register(SP) - 4);
                match mem.write_word(cpu.get_register(SP) as usize, cpu.get_register(reg as usize)) {
                    Ok(_) => (),
                    Err(e) => warn!("{}", e),
                };
            }
        }

        // NOTE: This only read from the SP so it's always a cycle per entry of 1
        let n = registers.len() as u32;
        if self.l {
            if registers.contains(&(PC as u32)) {
                // NOTE (n+1)S + 2N + 1I when PC is in register_list 
                cpu.add_cycles(n + 4)
            } else {
                // NOTE nS + 1N + 1I
                cpu.add_cycles(n + 2)
            }
        } else {
            // NOTE: (n-1)S + 2N
            cpu.add_cycles(n + 1)
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
        // TODO: use bit_map_to_array function here?
        for i in 0..8 {
            if (self.rlist >> i & 1) == 0 {
                continue;
            }

            if self.l {
                let value = match mem.read_word(cpu.get_register(self.rb) as usize) {
                    Ok(n) => n,
                    Err(e) => {
                        warn!("{}", e);
                        0
                    }
                };
                cpu.set_register(i, value);
                cpu.set_register(self.rb, cpu.get_register(self.rb) + 4);
            } else {
                match mem.write_word(cpu.get_register(self.rb) as usize, cpu.get_register(i)) {
                    Ok(_) => (),
                    Err(e) => warn!("{}", e),
                };
                cpu.set_register(self.rb, cpu.get_register(self.rb) + 4);
            }
        }

        let registers = bit_map_to_array(self.rlist.into());
        let n = registers.len() as u32;
        if self.l {
            if registers.contains(&(PC as u32)) {
                // NOTE (n+1)S + 2N + 1I when PC is in register_list 
                cpu.add_cycles(n + 4)
            } else {
                // NOTE nS + 1N + 1I
                cpu.add_cycles(n + 2)
            }
        } else {
            // NOTE: (n-1)S + 2N
            cpu.add_cycles(n + 1)
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
            cond: match value >> 8 & 0xf {
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
            cpu.add_cycles(1);
            return;
        }

        let offset = if self.offset & (1 << 7) == (1 << 7) {
            ((self.offset << 1) | 0xfffffe00) as i32
        } else {
            (self.offset << 1) as i32
        };

        let offset_abs: u32 = u32::try_from(offset.abs()).unwrap_or(0);

        let addr = if offset < 0 {
            cpu.get_register(PC) - offset_abs
        } else {
            cpu.get_register(PC) + offset_abs
        };

        cpu.decode = match mem.read_halfword(addr as usize) {
            Ok(n) => n,
            Err(_) => 0,
        };
        cpu.inst_addr = addr as usize;

        cpu.set_register(PC, addr + 2);
        // NOTE: 3S + 1N
        cpu.add_cycles(3)
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
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        println!("{:?}", cpu);
        println!("CPU PC: {:x}", cpu.pc());
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
            (self.offset) | 0xfffff000
        } else {
            self.offset
        };
        let addr = cpu.get_register(PC).wrapping_add(offset);

        cpu.decode = match mem.read_halfword(addr as usize) {
            Ok(n) => n,
            Err(e) => {
                warn!("{}", e);
                0
            }
        };
        cpu.inst_addr = addr as usize;

        cpu.set_register(PC, addr + 2);
        // NOTE: 2S + 1N
        cpu.add_cycles(3);
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
    // NOTE: The cycles for this command are split in 2
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        // !self.h runs first, the next addr MUST be another LongBranchWithLinkOp
        // with self.h == true
        if !self.h {
            let offset = if self.offset >> 10 & 1 == 1 {
                (self.offset << 12) | 0xff800000
            } else {
                self.offset << 12
            };
            let res = cpu.get_register(PC).wrapping_add(offset);

            cpu.set_register(LR, res);
            // NOTE: 3S + 1N
            cpu.add_cycles(1)
        } else {
            let temp = (cpu.get_register(PC) - 2) | 1;
            let res = cpu.get_register(LR).wrapping_add(self.offset << 1);
            cpu.set_register(PC, res);
            cpu.set_register(LR, temp);

            cpu.decode = match mem.read_halfword(cpu.get_register(PC) as usize) {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    0
                }
            };
            cpu.inst_addr = res as usize;

            cpu.set_register(PC, cpu.get_register(PC) + 2);
            // NOTE: 3S + 1N
            cpu.add_cycles(3);
        }
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
        let op = AddSubtractOp::from(inst);
        assert_eq!(op, AddSubtractOp{rd: 5, rs: 5, rn: 6, i: false, op: false, offset: 6});
    }

    #[test]
    fn test_sub_imm_variant() {
        let inst: u32 = 0x1e68;
        let op = AddSubtractOp::from(inst);
        assert_eq!(op, AddSubtractOp{rd: 0, rs: 5, rn: 1, i: true, op: true, offset: 1});
    }

    #[test]
    fn test_add_imm_variant() {
        let inst: u32 = 0x1c22;
        let op = AddSubtractOp::from(inst);
        assert_eq!(op, AddSubtractOp{rd: 2, rs: 4, offset: 0, rn: 0, i: true, op: false});
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
        assert_eq!(op, HiRegOp{h1: false, h2: true, rd: 0, rs: 14, op: 3});
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
