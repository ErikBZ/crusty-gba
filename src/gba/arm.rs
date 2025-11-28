use crate::utils::Bitable;
use crate::utils::shifter::CpuShifter;
use super::system::{read_cycles_per_32, read_cycles_per_8_16};
use super::{Operation, SystemMemory, get_v_from_sub, get_v_from_add, bit_map_to_array};
use super::{CPSR_C, CPSR_T};
use super::cpu::{Cpu,PC, LR};
use tracing::warn;
use super::utils::calc_cycles_for_stm_ldm;
use super::error::InstructionDecodeError;

// TODO: currently all regs are u8 or u32 types, maybe they should be usizes
pub fn decode_as_arm(inst: u32) -> Result<Box<dyn Operation>, InstructionDecodeError> {
    if is_multiply(inst) {
       Ok(Box::new(MultiplyOp::from(inst)))
    } else if is_multiply_long(inst) {
       Ok(Box::new(MultiplyLongOp::from(inst)))
    } else if is_single_data_swap(inst) {
       Ok(Box::new(SingleDataSwapOp::from(inst)))
    } else if is_branch_and_exchange(inst) {
       Ok(Box::new(BranchExchangeOp::from(inst)))
    } else if is_branch(inst) {
       Ok(Box::new(BranchOp::from(inst)))
    } else if is_software_interrupt(inst) {
        Ok(Box::new(SoftwareInterruptOp))
    } else if is_single_data_tfx(inst) {
       Ok(Box::new(SingleDataTfx::from(inst)))
    } else if is_block_data_tfx(inst) {
       Ok(Box::new(BlockDataTransfer::from(inst)))
    } else if is_coprocessor_data_op(inst) {
       Ok(Box::new(CoprocessDataOp::from(inst)))
    } else if is_coprocessor_data_tfx(inst) {
       Ok(Box::new(CoprocessDataTfx::from(inst)))
    } else if is_coprocessor_reg_tfx(inst) {
       Ok(Box::new(CoprocessRegTfx::from(inst)))
    } else if is_psr_transfer(inst) {
       Ok(Box::new(PsrTransferOp::from(inst)))
    } else if is_halfword_data_tfx_imm(inst) || is_halfword_data_tfx_reg(inst) {
       Ok(Box::new(HalfwordDataOp::from(inst)))
    } else if is_data_processing(inst) {
       Ok(Box::new(DataProcessingOp::from(inst)))
    } else {
       Ok(Box::new(UndefinedInstruction))
    }
}

#[derive(Debug)]
struct UndefinedInstruction;
impl Operation for UndefinedInstruction {
    // TODO: Implement. Take undef trap
    // TODO: Track Cycles 2S + 1I + 1N
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        unreachable!()
    }
}

#[derive(Debug)]
struct SoftwareInterruptOp;
impl Operation for SoftwareInterruptOp {
    // TODO: Implement
    // TODO: Track Cycles 2S + 1N
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum AddressingMode3 {
    Imm(u8),
    Reg(u8),
}

// TODO: Maybe rename this to DataOperation and use other structs
// like branch operation
#[derive(Debug, PartialEq)]
pub struct DataProcessingOp {
    pub s: bool,
    pub i: bool,
    pub rn: u8,
    pub rd: u8,
    operand: Operand,
    opcode: DataProcessingType,
}

#[derive(Debug, PartialEq)]
enum DataProcessingType {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Orr,
    Mov,
    Bic,
    Mvn,
}

impl From<u32> for DataProcessingOp {
    fn from(inst: u32) -> Self {
        let opcode = match inst >> 21 & 0xf {
            0 => DataProcessingType::And,
            1 => DataProcessingType::Eor,
            2 => DataProcessingType::Sub,
            3 => DataProcessingType::Rsb,
            4 => DataProcessingType::Add,
            5 => DataProcessingType::Adc,
            6 => DataProcessingType::Sbc,
            7 => DataProcessingType::Rsc,
            8 => DataProcessingType::Tst,
            9 => DataProcessingType::Teq,
            10 => DataProcessingType::Cmp,
            11 => DataProcessingType::Cmn,
            12 => DataProcessingType::Orr,
            13 => DataProcessingType::Mov,
            14 => DataProcessingType::Bic,
            _ => DataProcessingType::Mvn,
        };
        DataProcessingOp {
            i: (inst >> 25 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd: (inst >> 12 & 0xf) as u8,
            rn: (inst >> 16 & 0xf) as u8,
            operand: Operand::from(inst),
            opcode
        }
    }
}

impl Operation for DataProcessingOp {
    fn run(&self, cpu: &mut Cpu, _mem: &mut SystemMemory) {
        let (op2, c_out) = self.operand.apply(cpu);
        // NOTE: check the operand type. Imm is 0, Reg is 1
        let cycle = 1;
        let mut cycles = 1;
        let rn_value = cpu.get_register(self.rn as usize) as u64;

        cycles += cycle;

        if self.rd as usize == PC {
            cycles += 1;
        }

        let operand2 = op2 as u64;
        let carry = ((cpu.cpsr & CPSR_C) >> 29) as u64;
        let mut v_status = false;

        let res = match self.opcode {
            DataProcessingType::And | DataProcessingType::Tst => {
                rn_value & operand2
            },
            DataProcessingType::Eor | DataProcessingType::Teq => {
                (rn_value ^ operand2) & 0xffffffff
            },
            DataProcessingType::Sub | DataProcessingType::Cmp => {
                // Note: 2s complementing
                let rhs = !op2 as u64;
                let res = rn_value + rhs + 1;
                v_status = get_v_from_sub(rn_value, operand2, res);
                res
            },
            DataProcessingType::Rsb => {
                // Note: 2s complementing
                let rhs = !cpu.get_register(self.rn as usize) as u64;
                let res = operand2 + rhs + 1;
                v_status = get_v_from_sub(operand2, rn_value, res);
                res
            },
            DataProcessingType::Add | DataProcessingType::Cmn => {
                let res = rn_value + operand2;
                v_status = get_v_from_add(rn_value, operand2, res);
                res
            },
            DataProcessingType::Adc => {
                let res = rn_value + operand2 + carry;
                v_status = get_v_from_add(rn_value, operand2, res);
                res
            },
            // TODO: Ccheck v_staus here:
            DataProcessingType::Sbc => {
                // Note: 2s complementing
                let rhs = !op2 as u64;
                let res = rn_value + rhs + carry;
                v_status = get_v_from_sub(rn_value, operand2, res);
                res
            },
            DataProcessingType::Rsc => {
                let rhs = !cpu.get_register(self.rn as usize) as u64;
                let res = operand2 + rhs + carry;
                v_status = get_v_from_sub(operand2, rn_value, res);
                res
            },
            DataProcessingType::Orr => {
                rn_value | operand2
            },
            DataProcessingType::Mov => {
                operand2
            },
            DataProcessingType::Bic => {
                rn_value & !operand2
            }
            DataProcessingType::Mvn => {
                !operand2
            }
        };
        let c_out = if self.is_logical_operation() { c_out } else { res.bit_is_high(32) };
        let res: u32 = (res & 0xffffffff) as u32;

        if !(self.opcode == DataProcessingType::Cmp || self.opcode == DataProcessingType::Tst ||
            self.opcode == DataProcessingType::Teq || self.opcode == DataProcessingType::Cmn) {
            cpu.set_register(self.rd as usize, res);
        }

        if matches!(self.opcode,
            DataProcessingType::And | DataProcessingType::Eor | DataProcessingType::Tst | DataProcessingType::Teq |
            DataProcessingType::Orr | DataProcessingType::Mov | DataProcessingType::Bic | DataProcessingType::Mvn) {
            v_status = cpu.v_status();
        }

        if self.s {
            cpu.update_cpsr(res, v_status, c_out);
        }
        cpu.add_cycles(cycles);
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
}

impl From<u32> for ShiftType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Lsl,
            1 => Self::Lsr,
            2 => Self::Asr,
            3 => Self::Ror,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, PartialEq)]
enum Operand {
    // u32 ror u32 * 2
    Imm (u32, u32, ShiftType),
    /// reg[usize] shift_by reg[usize]
    ShiftWithReg (usize, usize, ShiftType),
    /// reg[usize] shift_by u32
    ShiftImm (usize, u32, ShiftType),
}

// TODO: Try this instead? A bit cleaner:
// enum ArmValue {
//     Imm(u32),
//     Reg(usize)
// }
//
// struct Operand {
//     lhs: ArmValue,
//     rhs: ArmValue,
//     shift_type: ShiftType,
// }

impl From<u32> for Operand {
    fn from(value: u32) -> Self {
        if value.bit_is_high(25) {
            let rot = (value >> 8) & 0xf;
            let op = value & 0xff;
            Self::Imm (op, rot * 2, ShiftType::Ror)
        } else {
            let rm = (value & 0xf) as usize;
            let shift_type = ShiftType::from((value >> 5) & 0b11);
            if value.bit_is_high(4) {
                let reg = ((value >> 8) & 0xf) as usize;
                Self::ShiftWithReg(rm, reg, shift_type)
            } else {
                let val = (value >> 7) & 0x1f;
                Self::ShiftImm(rm, val, shift_type)
            }
        }
    }
}

impl Operand {
    fn lhs(&self, cpu: &Cpu) -> u32 {
        match self {
            Self::Imm(x, _, _) => *x,
            Self::ShiftWithReg(x, _, _) => cpu.get_register(*x),
            Self::ShiftImm(x, _, _) => cpu.get_register(*x),
        }
    }

    fn rhs(&self, cpu: &Cpu) -> u32 {
        match self {
            Self::Imm(_, y, _) => *y,
            Self::ShiftWithReg(_, y, _) => cpu.get_register(*y),
            Self::ShiftImm(_, y, _) => *y,
        }
    }

    fn shift(&self) -> ShiftType {
        match self {
            Self::Imm(_, _, sh_type) => *sh_type,
            Self::ShiftWithReg(_, _, sh_type) => *sh_type,
            Self::ShiftImm(_, _, sh_type) => *sh_type,
        }
    }
    /// Returns (value, cycles, carry_out)
    //NOTE: The assembler will convert LSR #0, ROR #0, ASR #0 to LSL #0,
    //      so I shouldn't have to do anything
    fn apply(&self, cpu: &Cpu) -> (u32, bool) {
        let lhs = self.lhs(cpu);
        //NOTE: Seems all my functions already handle the special cases, except rrx
        //      Maybe use them directly here?
        // ASR #0 encodes ASR #32 but we already handle that in the function
        // LSR #32
        if let Operand::ShiftImm(_, 0, ShiftType::Lsr) = self {
            return cpu.shr_with_carry(lhs, 32);
        // RRX
        } else if let Operand::ShiftImm(_, 0, ShiftType::Ror) = self {
            return cpu.rrx_with_carry(lhs);
        }

        let rhs = self.rhs(cpu);
        let shift_type = self.shift();
        let (res, c_out) = match shift_type {
            ShiftType::Lsl => {
                cpu.shl_with_carry(lhs, rhs)
            },
            ShiftType::Lsr => {
                cpu.shr_with_carry(lhs, rhs)
            },
            ShiftType::Asr => {
                cpu.asr_with_carry(lhs, rhs)
            },
            ShiftType::Ror => {
                cpu.ror_with_carry(lhs, rhs)
            },
        };

        (res, c_out)
    }
}


impl DataProcessingOp {
    fn is_logical_operation(&self) -> bool {
        self.opcode == DataProcessingType::And ||
        self.opcode == DataProcessingType::Eor ||
        self.opcode == DataProcessingType::Tst ||
        self.opcode == DataProcessingType::Teq ||
        self.opcode == DataProcessingType::Orr ||
        self.opcode == DataProcessingType::Mov ||
        self.opcode == DataProcessingType::Bic ||
        self.opcode == DataProcessingType::Mvn
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiplyOp {
    // whether operation should accumulate
    a: bool,
    s: bool,
    rd: u8,
    rn: u8,
    rs: u8,
    rm: u8,
}

impl Operation for MultiplyOp {
    fn run(&self, cpu: &mut Cpu, _mem: &mut SystemMemory) {
        let rn_value = cpu.get_register(self.rn as usize);
        let rs_value = cpu.get_register(self.rs as usize);
        let rm_value = cpu.get_register(self.rm as usize);

        let mut res = rm_value.wrapping_mul(rs_value);
        if self.a {
            res += rn_value;
        }

        cpu.set_register(self.rd as usize, res);
        // TODO: C is meaningless and V is unaffected. Update this to reflect this
        cpu.update_cpsr(res, false, false);
        // NOTE:
        //      MUL: 1S +(m)I
        //      MLA: 1S +(m+1)I
        cpu.add_cycles(self.count_cycles(rs_value));
    }
}

impl MultiplyOp {
    fn count_cycles(&self, mult_operand: u32) -> u32 {
        let mut m = if (mult_operand & 0xffffff00) == 0 || (mult_operand & 0xffffff00 == 0xffffff00) {
            1
        } else if (mult_operand & 0xffff0000) == 0 || (mult_operand & 0xffff0000 == 0xffff0000) {
            2
        } else if (mult_operand & 0xff000000) == 0 || (mult_operand & 0xff000000 == 0xff000000) {
            3
        } else {
            4
        };

        if self.a {
            m += 1;
        }

        1 + m
    }
}

impl From<u32> for MultiplyOp {
    fn from(inst: u32) -> Self {
        Self {
            a: (inst >> 21 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd: (inst >> 16 & 0xf) as u8,
            rn: (inst >> 12 & 0xf) as u8,
            rs: (inst >> 8 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MultiplyLongOp {
    // whether operation is signed or unsigned
    u: bool,
    // whether opeartion should accumulate
    a: bool,
    s: bool,
    rd_hi: u8,
    rd_lo: u8,
    rs: u8,
    rm: u8,
}

impl Operation for MultiplyLongOp {
    // TODO: Implement
    // TODO: Track Cycles
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        todo!()
    }
}

impl From<u32> for MultiplyLongOp {
    fn from(inst: u32) -> Self {
        Self {
            u: (inst >> 22 & 0x1) == 0x1,
            a: (inst >> 21 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd_hi: (inst >> 16 & 0xf) as u8,
            rd_lo: (inst >> 12 & 0xf) as u8,
            rs: (inst >> 8 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SingleDataSwapOp {
    b: bool,
    rn: u8,
    rd: u8,
    rm: u8,
}

impl Operation for SingleDataSwapOp {
    // TODO: Propogate Error for ABORT signals
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        let address = cpu.get_register(self.rn as usize) as usize;
        match mem.read_from_mem(address) {
            Ok(n) => cpu.set_register(self.rd as usize, n),
            Err(e) => warn!("{}", e),
        }

        let data = cpu.get_register(self.rm as usize);
        let cycles = if self.b {
            match mem.write_byte(address, data) {
                Ok(_) => (),
                Err(e) => warn!("{}", e),
            };
            read_cycles_per_8_16(address)
        } else {
            match mem.write_word(address, data) {
                Ok(_) => (),
                Err(e) => warn!("{}", e),
            };
            read_cycles_per_32(address)
        };

        // TODO: 1S + 2N + 1I
        cpu.add_cycles(cycles + 3);
    }
}

impl From<u32> for SingleDataSwapOp {
    fn from(inst: u32) -> Self {
        Self {
            b: (inst >> 22 & 0x1) == 0x1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BranchExchangeOp {
    rn: u8,
}

impl Operation for BranchExchangeOp {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        let mut addr = cpu.get_register(self.rn as usize);
        cpu.update_thumb(addr & 1 == 1);
        addr &= !1;
        // Pipeline flush
        cpu.decode = if cpu.is_thumb_mode() {
            match mem.read_halfword(addr as usize) {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    0
                }
            }
        } else {
            match mem.read_word(addr as usize) {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    0
                }
            }
        };

        cpu.inst_addr = addr as usize;
        if cpu.cpsr & CPSR_T == CPSR_T {
            cpu.set_register(PC, addr + 2);
        } else {
            cpu.set_register(PC, addr + 4);
        }

        // NOTE: 2S + 1N
        cpu.add_cycles(3);
    }
}

impl From<u32> for BranchExchangeOp {
    fn from(inst: u32) -> Self {
        Self {
            rn: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BranchOp {
    l: bool,
    offset: u32,
}

impl Operation for BranchOp {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        let offset = self.get_offset();
        let addr = cpu.get_register(PC).wrapping_add(offset);

        if self.l {
            // NOTE: LR has to be the current decode
            cpu.set_register(LR, cpu.get_register(PC) - 4);
        }

        cpu.decode = match mem.read_from_mem(addr as usize) {
            Ok(n) => n,
            Err(e) => {
                warn!("{}", e);
                0
            },
        };
        cpu.inst_addr = addr as usize;

        cpu.set_register(PC, addr + 4);
        // NOTE: 2S + 1N
        cpu.add_cycles(3);
    }
}

impl From<u32> for BranchOp {
    fn from(inst: u32) -> Self {
        Self {
            l: (inst >> 24 & 0x1) == 0x1,
            offset: (inst & 0xffffff) as u32,
        }
    }
}

impl BranchOp {
    pub fn get_offset(&self) -> u32 {
        // offset is shifted left by 2, and then sign extended to 32 bits
        if self.offset & (1 << 23) == (1 << 23) {
            (self.offset << 2) | 0xffc00000
        } else {
            self.offset << 2
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HalfwordRegOffset {
    p: bool,
    u: bool,
    w: bool,
    l: bool,
    s: bool,
    h: bool,
    rn: u8,
    rd: u8,
    rm: u8,
}

impl Operation for HalfwordRegOffset {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        let offset = cpu.get_register(self.rm as usize);
        let mut address = cpu.get_register(self.rn as usize);

        if self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }

            if self.w {
                cpu.set_register(self.rn as usize, address);
            }
        }

        let cycles_per_entry = read_cycles_per_8_16(address as usize);

        if self.l {
            cpu.add_cycles(cycles_per_entry + 3);
            let res = match mem.read_from_mem(address as usize) {
                Ok(n) => n,
                Err(e) => {
                    //TODO: Better error handling
                    warn!("{}", e);
                    0
                }
            };

            // TODO: Make this look nicer
            let value: u32 = match (self.h, self.s) {
                // TODO: Take into consideration the Endianess
                // LDRB handled by SingleDataTfx
                (false, false) => unreachable!(),
                // LDRSB
                (false, true) => {
                    if res & 0x80 == 0x80 {
                        res | 0xffffff00
                    } else {
                        res & 0xff
                    }
                },
                // LDRH
                (true, false) => {0},
                // LDRSH
                (true, true) => {
                    if res & 0x8000 == 0x8000 {
                        res | 0xffff0000
                    } else {
                        res & 0xffff
                    }
                }
            };

            cpu.set_register(self.rd as usize, value);
        } else {
            cpu.add_cycles(cycles_per_entry + 1);
            match mem.write_halfword(address as usize, cpu.get_register(self.rd as usize)) {
                Ok(_) => (),
                Err(e) => warn!("{}", e),
            }
        }

        if !self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }
            cpu.set_register(self.rn as usize, address);
        }
    }
}

impl From<u32> for HalfwordRegOffset {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            s: (inst >> 6 & 1) == 1,
            h: (inst >> 5 & 1) == 1,
            rn: (inst >> 15 & 0xf) as u8,
            rd: (inst >> 11 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SingleDataTfx {
    pub i: bool,
    pub p: bool,
    pub u: bool,
    pub b: bool,
    pub w: bool,
    pub l: bool,
    pub rn: u8,
    pub rd: u8,
    pub offset: u16,
}

impl Operation for SingleDataTfx {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        // TODO: add write back check somewhere
        let offset = self.get_offset(cpu);
        let mut tfx_add = cpu.get_register(self.rn as usize);

        if self.p {
            if self.u {
                tfx_add += offset;
            } else {
                tfx_add -= offset;
            }
            if self.w{
                cpu.set_register(self.rn as usize, tfx_add);
            }
        }

        let mut cycles = 0;

        if self.l {
            let data_block = if self.b {
                cycles += read_cycles_per_8_16(tfx_add as usize);
                mem.read_byte(tfx_add as usize)
            } else {
                cycles += read_cycles_per_32(tfx_add as usize);
                mem.read_word(tfx_add as usize)
            };

            let res = match data_block {
                Ok(n) => n,
                Err(e) => {
                    warn!("{}", e);
                    panic!()
                },
            };
            cpu.set_register(self.rd as usize, res);
            if self.rd as usize == PC {
                // NOTE: 2S + 2N + 1I
                cpu.add_cycles(cycles + 4);
            } else {
                // NOTE: 1S + 1N + 1I
                cpu.add_cycles(cycles + 2);
            }
        } else {
            // NOTE: 2N
            let res = if self.b {
                cycles += read_cycles_per_8_16(tfx_add as usize);
                mem.write_byte(tfx_add as usize, cpu.get_register(self.rd as usize))
            } else {
                cycles += read_cycles_per_32(tfx_add as usize);
                mem.write_word(tfx_add as usize, cpu.get_register(self.rd as usize))
            };

            cpu.add_cycles(cycles + 1);

            match res {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e)
                }
            }
        }

        // NOTE: for L i don't think this matters
        // for LDR
        if !self.p {
            if self.u {
                tfx_add += offset;
            } else {
                tfx_add -= offset;
            }
            cpu.set_register(self.rn as usize, tfx_add);
        }
    }
}

impl From<u32> for SingleDataTfx {
    fn from(inst: u32) -> Self {
        Self {
            i: (inst >> 25 & 1) == 1,
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            b: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            offset: (inst & 0xfff) as u16,
        }
    }
}

impl SingleDataTfx {
    pub fn get_offset(&self, cpu: &Cpu) -> u32 {
        if self.i {
            let shift = (self.offset >> 4) & 0xff;
            (cpu.get_register((self.offset & 0xf) as usize) << shift) as u32
        } else {
            self.offset as u32
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BlockDataTransfer {
    p: bool,
    u: bool,
    s: bool,
    w: bool,
    l: bool,
    rn: u8,
    register_list: u16,
}

impl Operation for BlockDataTransfer {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        // TODO: Take into consideration the S flag
        // TODO: Propogate the mem error to signify ABORT signal
        // When rn is 13 then we are doing stack ops, otherwise no
        let mut address = cpu.get_register(self.rn as usize) as usize;
        let registers = bit_map_to_array(self.register_list as u32);

        for i in 0..registers.len() {
            if self.p && self.w {
                if self.u {
                    address += 4
                } else {
                    address -= 4
                }
                cpu.set_register(self.rn as usize, address as u32);
            }

            let reg = if !self.u {
                registers.len() - i - 1
            } else {
                i
            };

            if self.l {
                let res = match mem.read_from_mem(address){
                    Ok(b) => b,
                    Err(e) => {
                        warn!("{}", e);
                        0
                    }
                };
                cpu.set_register(registers[reg] as usize, res);
            } else {
                match mem.write_word(address, cpu.get_register(registers[reg] as usize)) {
                    Ok(_) => (),
                    Err(e) => {
                        warn!("{}", e);
                    }
                };
            }

            if !self.p && self.w {
                if self.u {
                    address += 4
                } else {
                    address -= 4
                }
                cpu.set_register(self.rn as usize, address as u32);
            }
        }

        let entries = registers.len() as u32;
        let cycles_per_entry = read_cycles_per_32(address);
        let cycles = calc_cycles_for_stm_ldm(cycles_per_entry, entries, self.l, registers.contains(&(PC as u32)));
        cpu.add_cycles(cycles);
    }
}

impl From<u32> for BlockDataTransfer {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            s: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            register_list: (inst & 0xffff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessDataTfx {
    p: bool,
    u: bool,
    n: bool,
    w: bool,
    l: bool,
    rn: u8,
    c_rd: u8,
    cp_num: u8,
    offset: u16,
}

impl Operation for CoprocessDataTfx {
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        todo!()
    }
}

impl From<u32> for CoprocessDataTfx {
    fn from(inst: u32) -> Self {
        Self {
            p: (inst >> 24 & 1) == 1,
            u: (inst >> 23 & 1) == 1,
            n: (inst >> 22 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            offset: (inst & 0xffff) as u16,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessDataOp {
    cp_opc: u8,
    c_rn: u8,
    c_rd: u8,
    cp_num: u8,
    cp: u8,
    c_rm: u8,
}

impl Operation for CoprocessDataOp {
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        todo!()
    }
}

impl From<u32> for CoprocessDataOp {
    fn from(inst: u32) -> Self {
        Self {
            cp_opc: (inst >> 20 & 0xf) as u8,
            c_rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            cp: (inst >> 5 & 0x7) as u8,
            c_rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CoprocessRegTfx {
    l: bool,
    cp_opc: u8,
    c_rn: u8,
    c_rd: u8,
    cp_num: u8,
    cp: u8,
    c_rm: u8,
}

impl Operation for CoprocessRegTfx {
    fn run(&self, _cpu: &mut Cpu, _mem: &mut SystemMemory) {
        todo!()
    }
}

impl From<u32> for CoprocessRegTfx {
    fn from(inst: u32) -> Self {
        Self {
            l: (inst >> 20 & 1) == 1,
            cp_opc: (inst >> 21 & 0xf) as u8,
            c_rn: (inst >> 16 & 0xf) as u8,
            c_rd: (inst >> 12 & 0xf) as u8,
            cp_num: (inst >> 8 & 0xf) as u8,
            cp: (inst >> 5 & 0x7) as u8,
            c_rm: (inst & 0xf) as u8,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PsrTransferOp {
    i: bool,
    p: bool,
    bit_flags_only: bool,
    pub rd: u8,
    rm: u8,
    rotate: u8,
    imm: u8,
    op: PsrTransferType,
}

#[derive(Debug, PartialEq)]
enum PsrTransferType {
    MSR,
    MRS
}

impl Operation for PsrTransferOp {
    fn run(&self, cpu: &mut Cpu, _mem: &mut SystemMemory) {
        match self.op {
            PsrTransferType::MSR => {
                let operand = self.get_operand(cpu);
                let mask: u32 = if self.is_bit_flag_only() {
                    0xf0000000
                } else {
                    0xffffffff
                };

                if self.is_cspr() {
                    cpu.cpsr = (cpu.cpsr & !mask) | (operand & mask)
                } else {
                    cpu.set_psr(cpu.get_psr() & !mask | (operand & mask))
                }
            },
            PsrTransferType::MRS => {
                if self.is_cspr() {
                    cpu.set_register(self.rd as usize, cpu.cpsr);
                } else {
                    cpu.set_register(self.rd as usize, cpu.get_psr());
                }
            },
        }
        // NOTE: (MSR, MRS) 1S
        cpu.add_cycles(1)
    }
}

impl From<u32> for PsrTransferOp {

    fn from(inst: u32) -> Self {
        let op = if is_mrs_op(inst) {
            PsrTransferType::MRS
        } else {
            PsrTransferType::MSR
        };

        Self {
            i: (inst >> 25 & 1) == 1,
            p: (inst >> 22 & 1) == 1,
            bit_flags_only: is_psr_flag_bits_only(inst),
            rd: (inst >> 12 & 0xf) as u8,
            rm: (inst & 0xf) as u8,
            rotate: (inst >> 8 & 0xf) as u8,
            imm: (inst & 0xff) as u8,
            op
        }
    }
}

impl PsrTransferOp {
    pub fn get_operand(&self, cpu: &Cpu) -> u32 {
        if self.i {
            let imm = self.imm as u32;
            imm.rotate_right((self.rotate as u32) * 2)
        } else {
            cpu.get_register(self.rm as usize)
        }
    }

    pub fn is_cspr(&self) -> bool {
        !self.p
    }

    pub fn is_bit_flag_only(&self) -> bool {
        self.bit_flags_only
    }
}

#[derive(Debug, PartialEq)]
pub struct HalfwordDataOp {
    mode: AddressingMode3,
    p: bool,
    u: bool,
    w: bool,
    l: bool,
    s: bool,
    h: bool,
    rn: u8,
    rd: u8,
}

impl Operation for HalfwordDataOp {
    fn run(&self, cpu: &mut Cpu, mem: &mut SystemMemory) {
        let offset = match self.mode {
            AddressingMode3::Reg(m) => cpu.get_register(m as usize),
            AddressingMode3::Imm(byte_offset) => byte_offset as u32,
        };
        let mut address = cpu.get_register(self.rn as usize);

        if self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }

            if self.w {
                cpu.set_register(self.rn as usize, address);
            }
        }

        let cycles_per_entry = read_cycles_per_8_16(address as usize);

        if self.l {
            let res = match mem.read_from_mem(address as usize) {
                Ok(n) => n,
                Err(e) => {
                    //TODO: Better error handling
                    warn!("{}", e);
                    0
                }
            };

            let value: u32 = match (self.h, self.s) {
                // TODO: Take into consideration the Endianess
                // LDRB handled by SingleDataTfx
                (false, false) => unreachable!(),
                // LDRSB
                (false, true) => {
                    if res & 0x80 == 0x80 {
                        res | 0xffffff00
                    } else {
                        res & 0xff
                    }
                },
                // LDRH
                (true, false) => {
                    res & 0xffff
                },
                // LDRSH
                (true, true) => {
                    if res & 0x8000 == 0x8000 {
                        res | 0xffff0000
                    } else {
                        res & 0xffff
                    }
                }
            };

            cpu.set_register(self.rd as usize, value);
            if self.rd as usize == PC {
                // NOTE: 2I + 2N 1I
                cpu.add_cycles(cycles_per_entry + 4);
            } else {
                // NOTE: 1I + 1N 1I
                cpu.add_cycles(cycles_per_entry + 2);
            }
        } else {
            if self.s || !self.h{
                unreachable!();
            };
            // STRH
            match mem.write_halfword(address as usize, cpu.get_register(self.rd as usize)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("{}", e);
                }
            }
            // NOTE: 2N
            cpu.add_cycles(cycles_per_entry + 1);
        }

        if !self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }
            cpu.set_register(self.rn as usize, address);
        }
    }
}

impl From<u32> for HalfwordDataOp {
    fn from(inst: u32) -> Self {
        let p = (inst >> 24 & 1) == 1;
        let byte_offset = ((inst & 0xf) | (inst >> 4 & 0xf0)) as u8;
        let rm = (inst & 0xf) as u8;

        // oh don't need post and pre
        let mode = if is_halfword_data_tfx_imm(inst) {
            AddressingMode3::Imm(byte_offset)
        } else {
            AddressingMode3::Reg(rm)
        };

        Self {
            p,
            u: (inst >> 23 & 1) == 1,
            w: (inst >> 21 & 1) == 1,
            l: (inst >> 20 & 1) == 1,
            s: (inst >> 6 & 1) == 1,
            h: (inst >> 5 & 1) == 1,
            rn: (inst >> 16 & 0xf) as u8,
            rd: (inst >> 12 & 0xf) as u8,
            mode,
        }
    }
}

// NOTE: this may match PSR Transfers so PSR transfers should be matched first
pub fn is_data_processing(inst: u32) -> bool {
    inst & 0x0c000000 == 0x00000000
}

pub fn is_multiply(inst: u32) -> bool {
    inst & 0x0fc000f0 == 0x00000090
}

pub fn is_multiply_long(inst: u32) -> bool {
    inst & 0x0fc000f0 == 0x00800090
}

pub fn is_single_data_swap(inst: u32) -> bool {
    inst & 0x0fb00ff0 == 0x01000090
}

pub fn is_branch_and_exchange(inst: u32) -> bool {
    inst & 0x0ffffff0 == 0x012fff10
}

pub fn is_halfword_data_tfx_reg(inst: u32) -> bool {
   inst & 0x0e400f90 == 0x00000090
}

pub fn is_halfword_data_tfx_imm(inst: u32) -> bool {
    inst & 0x0e400090 == 0x00400090
}

pub fn is_single_data_tfx(inst: u32) -> bool {
    inst & 0x0c000000 == 0x04000000
}

pub fn is_block_data_tfx(inst: u32) -> bool {
    inst & 0x0e000000 == 0x08000000
}

pub fn is_branch(inst: u32) -> bool {
    inst & 0x0e000000 == 0x0a000000
}

pub fn is_coprocessor_data_tfx(inst: u32) -> bool {
    inst & 0x0e000000 == 0x0c000000
}

pub fn is_coprocessor_data_op(inst: u32) -> bool {
    inst & 0x0f000010 == 0x0c000000
}

pub fn is_coprocessor_reg_tfx(inst: u32) -> bool {
    inst & 0x0f000010 == 0x0c000010
}

pub fn is_software_interrupt(inst: u32) -> bool {
    inst & 0x0f000000 == 0x0f000000
}

pub fn is_mrs_op(inst: u32) -> bool {
    inst & 0x0fbf0fff == 0x010f0000
}

pub fn is_psr_transfer(inst: u32) -> bool {
    (inst & 0x0fbf0fff == 0x010f0000) ||
    (inst & 0x0fbffff0 == 0x0129f000) ||
    (inst & 0x0dbff000 == 0x0128f000)
}

fn is_psr_flag_bits_only(inst: u32) -> bool {
    (inst & 0x0dbff000) == 0x0128f000
}

mod test {
    #![allow(unused)]
    use super::*;

    #[test]
    fn test_branch_check() {
        let inst: u32 = 0b11101010000000000000000000011000;
        let branch = is_branch(inst);
        assert_eq!(branch, true);
    }

    #[test]
    fn test_branch_decode() {
        let inst: u32 = 0b11101010000000000000000000011000;
        let op = BranchOp::from(inst);
        let op2 = BranchOp {
            l: false,
            offset: 0b11000,
        };

        assert_eq!(op, op2);
    }

    #[test]
    fn test_mrs_op_check() {
        let inst: u32 = 0xe14fc000;
        let is_mrs = is_mrs_op(inst);
        assert_eq!(is_mrs, true);
    }

    #[test]
    fn test_strb_decode() {
        let inst: u32 = 0xe5cc3301;
        let op = SingleDataTfx::from(inst);
        let op2 = SingleDataTfx {
            i: false,
            p: true,
            u: true,
            b: true,
            w: false,
            l: false,
            rn: 12,
            rd: 3,
            offset: 0x301,
        };
        assert_eq!(op, op2);
    }

    #[test]
    fn test_strh_decode() {
        let inst: u32 = 0xe08180b3;
        let op = HalfwordDataOp::from(inst);
        let op2 = HalfwordDataOp {
            p: false,
            u: true,
            w: false,
            l: false,
            h: true,
            s: false,
            rn: 1,
            rd: 8,
            mode: AddressingMode3::Reg(3),
        };
        assert_eq!(op, op2);
    }

    #[test]
    fn test_msreq_decode() {
        let inst: u32 = 0x0129f00c;
        let op = PsrTransferOp::from(inst);
        let op2 = PsrTransferOp{
            i: false,
            imm: 12,
            bit_flags_only: false,
            p: false,
            rd: 15,
            rm: 12,
            rotate: 0,
            op: PsrTransferType::MSR
        };
        assert_eq!(op, op2);
    }

    #[test]
    fn test_msreq_decode_2() {
        let inst: u32 = 0x010fc000;
        let op = PsrTransferOp::from(inst);
        let op2 = PsrTransferOp{
            i: false,
            imm: 0,
            bit_flags_only: false,
            p: false,
            rd: 12,
            rm: 0,
            rotate: 0,
            op: PsrTransferType::MRS
        };
        assert_eq!(op, op2);
    }

    #[test]
    fn test_teq_decode() {
        let inst: u32 = 0xe1300003;
        let op = DataProcessingOp::from(inst);
        let op2 = DataProcessingOp {
            opcode: DataProcessingType::Teq,
            i: false,
            operand: Operand::ShiftWithReg(0, 3, ShiftType::Lsr),
            s: true,
            rn: 0,
            rd: 0
        };
        assert_eq!(op, op2);
    }
}
