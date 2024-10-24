use super::cpu::CPU;
use super::Operation;
use super::SystemMemory;
use super::CPSR_C;

use super::cpu::PC;
use super::CPSR_T;

// TODO: Possible alternative to this is to have all 15
// operation structs be a trait "Operable", that takes a CPU
// and modifies it based on it's instruction

// TODO: currently all regs are u8 or u32 types, maybe they should be usizes

pub fn decode_as_arm(inst: u32) -> Box<dyn Operation> {
    if is_multiply(inst) {
       Box::new(MultiplyOp::from(inst))
    } else if is_multiply_long(inst) {
       Box::new(MultiplyLongOp::from(inst))
    } else if is_single_data_swap(inst) {
       Box::new(SingleDataSwapOp::from(inst))
    } else if is_branch_and_exchange(inst) {
       Box::new(BranchExchangeOp::from(inst))
    } else if is_branch(inst) {
       Box::new(BranchOp::from(inst))
    } else if is_software_interrupt(inst) {
        Box::new(SoftwareInterruptOp)
    } else if is_single_data_tfx(inst) {
       Box::new(SingleDataTfx::from(inst))
    } else if is_block_data_tfx(inst) {
       Box::new(BlockDataTransfer::from(inst))
    } else if is_coprocessor_data_op(inst) {
       Box::new(CoprocessDataOp::from(inst))
    } else if is_coprocessor_data_tfx(inst) {
       Box::new(CoprocessDataTfx::from(inst))
    } else if is_coprocessor_reg_tfx(inst) {
       Box::new(CoprocessRegTfx::from(inst))
    } else if is_psr_transfer(inst) {
       Box::new(PsrTransferOp::from(inst))
    } else if is_halfword_data_tfx_imm(inst) || is_halfword_data_tfx_reg(inst) {
       Box::new(HalfwordDataOp::from(inst))
    } else if is_data_processing(inst) {
       Box::new(DataProcessingOp::from(inst))
    } else {
       Box::new(UndefinedInstruction)
    }
}

#[derive(Debug)]
struct UndefinedInstruction;
impl Operation for UndefinedInstruction {
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
        unreachable!()
    }
}

#[derive(Debug)]
struct SoftwareInterruptOp;
impl Operation for SoftwareInterruptOp {
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
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
    pub operand: u32,
    opcode: DataProcessingType,
}

#[derive(Debug, PartialEq)]
enum DataProcessingType{
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

impl From<u32> for DataProcessingOp {
    fn from(inst: u32) -> Self {
        let opcode = match inst >> 21 & 0xf {
            0 => DataProcessingType::AND,
            1 => DataProcessingType::EOR,
            2 => DataProcessingType::SUB,
            3 => DataProcessingType::RSB,
            4 => DataProcessingType::ADD,
            5 => DataProcessingType::ADC,
            6 => DataProcessingType::SBC,
            7 => DataProcessingType::RSC,
            8 => DataProcessingType::TST,
            9 => DataProcessingType::TEQ,
            10 => DataProcessingType::CMP,
            11 => DataProcessingType::CMN,
            12 => DataProcessingType::ORR,
            13 => DataProcessingType::MOV,
            14 => DataProcessingType::BIC,
            _ => DataProcessingType::MVN,
        };
        DataProcessingOp {
            i: (inst >> 25 & 0x1) == 0x1,
            s: (inst >> 20 & 0x1) == 0x1,
            rd: (inst >> 12 & 0xf) as u8,
            rn: (inst >> 16 & 0xf) as u8,
            operand: (inst & 0xfff) as u32,
            opcode
        }
    }
}

impl Operation for DataProcessingOp {
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        let operand2 = self.get_operand2(cpu.registers);
        let rn_value = cpu.registers[self.rn as usize];
        let carry = (cpu.cpsr & CPSR_C) >> 29;

        let res = match self.opcode {
            DataProcessingType::AND | DataProcessingType::TST => {
                rn_value & operand2
            },
            DataProcessingType::EOR | DataProcessingType::TEQ => {
                rn_value ^ operand2
            },
            DataProcessingType::SUB | DataProcessingType::CMP => {
                rn_value - operand2
            },
            DataProcessingType::RSB => {
                operand2 - rn_value
            },
            DataProcessingType::ADD | DataProcessingType::CMN => {
                rn_value + operand2
            },
            DataProcessingType::ADC => {
                rn_value + operand2 + carry
            },
            DataProcessingType::SBC => {
                rn_value - operand2 + carry - 1
            },
            DataProcessingType::RSC => {
                operand2 - rn_value + carry - 1
            },
            DataProcessingType::ORR => {
                rn_value | operand2
            },
            DataProcessingType::MOV => {
                operand2
            },
            DataProcessingType::BIC => {
                rn_value & !operand2
            }
            DataProcessingType::MVN => {
                !operand2
            }
        };

        if !(self.opcode == DataProcessingType::CMP && self.opcode == DataProcessingType::TST &&
            self.opcode == DataProcessingType::TEQ && self.opcode == DataProcessingType::CMN) {
            cpu.registers[self.rd as usize] = res;
        }

        if self.s {
            cpu.update_cpsr(res);
        }
    }
}

impl DataProcessingOp {
    pub fn get_operand2(&self, registers: [u32;16]) -> u32 {
        if self.i {
            let rotate = (self.operand >> 8 & 0xf) as u32;
            let op = (self.operand & 0xff) as u32;
            // we gotta rotate by twice the amount
            op.rotate_right(rotate * 2)
        }
        else {
            let shift = (self.operand >> 4 & 0xff) as u32;

            let (s, s_type) = if shift & 1 == 1 {
                (registers[(shift >> 4) as usize], (shift >> 1) & 3)
            } else if shift & 1 == 0 {
                (shift >> 3, (shift >> 1) & 3)
            } else {
                unreachable!()
            };

            let rm = (self.operand & 0xf) as usize;

            // TODO: Change this to enum?
            match s_type {
                0 => registers[rm] << s,
                1 => registers[rm] >> s,
                2 => ((registers[rm] as i32) >> s) as u32,
                3 => registers[rm].rotate_right(s),
                _ => unreachable!()
            }
        }
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

impl Operation for MultiplyOp{
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        let rn_value = cpu.registers[self.rn as usize];
        let rs_value = cpu.registers[self.rs as usize];
        let rm_value = cpu.registers[self.rm as usize];

        let mut res = rm_value * rs_value;
        if self.a {
            res += rn_value;
        }

        cpu.registers[self.rd as usize] = res;
        cpu.update_cpsr(res)
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
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let address = cpu.registers[self.rn as usize] as usize;
        match mem.read_from_mem(address) {
            Ok(n) => cpu.registers[self.rd as usize] = n,
            Err(e) => println!("{}", e),
        }

        let data = cpu.registers[self.rm as usize];
        if self.b {
            match mem.write_byte(address, data) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        } else {
            match mem.write_word(address, data) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let mut addr = cpu.registers[self.rn as usize];
        cpu.update_thumb(addr & 1 == 1);
        addr &= !1;
        // Pipeline flush
        cpu.decode = match mem.read_from_mem(addr as usize) {
            Ok(n) => n,
            Err(e) => {
                println!("{}", e);
                0
            }
        };

        if cpu.cpsr & CPSR_T == CPSR_T {
            cpu.registers[PC] = addr + 2;
        } else {
            cpu.registers[PC] = addr + 4;
        }
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let offset = self.get_offset();
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

impl From<u32> for BranchOp {
    fn from(inst: u32) -> Self {
        Self {
            l: (inst >> 24 & 0x1) == 0x1,
            offset: (inst & 0xffffff) as u32,
        }
    }
}

impl BranchOp {
    pub fn get_offset(&self) -> i32 {
        // offset is shifted left by 2, and then sign extended to 32 bits
        if self.offset & (1 << 24) == (1 << 24) {
            ((self.offset << 2) | 0xffc00000) as i32
        } else {
            (self.offset << 2) as i32
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let offset = cpu.registers[self.rm as usize];
        let mut address = cpu.registers[self.rn as usize];

        if self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }

            if self.w {
                cpu.registers[self.rn as usize] = address;
            }
        }

        if self.l {
            let res = match mem.read_from_mem(address as usize) {
                Ok(n) => n,
                Err(e) => {
                    //TODO: Better error handling
                    println!("{}", e);
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

            cpu.registers[self.rd as usize] = value;
        } else {
            match mem.write_halfword(address as usize, cpu.registers[self.rd as usize]) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            }
        }

        if !self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }
            cpu.registers[self.rn as usize] = address;
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        // TODO: add write back check somewhere
        let offset = self.get_offset(cpu.registers);
        let mut tfx_add = cpu.registers[self.rn as usize];

        if self.p {
            if self.u {
                tfx_add += offset;
            } else {
                tfx_add -= offset;
            }
            if self.w{
                cpu.registers[self.rn as usize] = tfx_add;
            }
        }

        if self.l {
            let block_from_mem = match mem.read_from_mem(tfx_add as usize) {
                Ok(n) => n,
                Err(e) => {
                    println!("{}", e);
                    panic!()
                },
            };

            cpu.registers[self.rd as usize] = if self.b {
                block_from_mem
            } else {
                block_from_mem & 0xff
            };
        } else {
            if self.b {
                match mem.write_word(tfx_add as usize, cpu.registers[self.rd as usize]) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            } else {
                match mem.write_byte(tfx_add as usize, cpu.registers[self.rd as usize]) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            }
        }

        // NOTE: for L i don't think this matters
        // for LDR
        if !self.p & self.w{
            if self.u {
                tfx_add += offset;
            } else {
                tfx_add -= offset;
            }
            cpu.registers[self.rn as usize] = tfx_add;
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
    pub fn get_offset(&self, registers: [u32;16]) -> u32 {
        if self.i {
            let shift = (self.offset >> 4) & 0xff;
            (registers[(self.offset & 0xf) as usize] << shift) as u32
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        // TODO: Take into consideration the S flag
        // TODO: Propogate the mem error to signify ABORT signal
        // When rn is 13 then we are doing stack ops, otherwise no
        let mut address = cpu.registers[self.rn as usize] as usize;

        for i in 0..15 {
            if (self.register_list >> i & 1) == 0 {
                continue;
            }

            if self.p && self.w {
                if self.u {
                    address += 4
                } else {
                    address -= 4
                }
                cpu.registers[self.rn as usize] = address as u32;
            }

            if self.l {
                cpu.registers[i] = match mem.read_from_mem(address){
                    Ok(b) => b,
                    Err(e) => {
                        println!("{}", e);
                        0
                    }
                };
            } else {
                match mem.write_word(address, cpu.registers[i]) {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e);
                    }
                };
            }

            if !self.p && self.w {
                if self.u {
                    address += 4
                } else {
                    address -= 4
                }
                cpu.registers[self.rn as usize] = address as u32;
            }
        }
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
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
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
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
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
    fn run(&self, _cpu: &mut CPU, _mem: &mut SystemMemory) {
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
    fn run(&self, cpu: &mut CPU, _mem: &mut SystemMemory) {
        match self.op {
            PsrTransferType::MSR => {
                let operand = self.get_operand(cpu.registers);
                let mask: u32 = if self.is_bit_flag_only() {
                    0xf0000000
                } else {
                    0xffffffff
                };

                if self.is_cspr() {
                    cpu.cpsr = (cpu.cpsr & !mask) | (operand & mask)
                } else {
                    cpu.spsr = (cpu.spsr & !mask) | (operand & mask)
                }
            },
            PsrTransferType::MRS => {
                if self.is_cspr() {
                    cpu.registers[self.rd as usize] = cpu.cpsr;
                } else {
                    cpu.registers[self.rd as usize] = cpu.spsr;
                }
            },
        }
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
    pub fn get_operand(&self, registers: [u32;16]) -> u32 {
        if self.i {
            let imm = self.imm as u32;
            imm.rotate_right((self.rotate as u32) * 2)
        } else {
            registers[self.rm as usize]
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
    fn run(&self, cpu: &mut CPU, mem: &mut SystemMemory) {
        let offset = match self.mode {
            AddressingMode3::Reg(m) => cpu.registers[m as usize],
            AddressingMode3::Imm(byte_offset) => byte_offset as u32,
        };
        let mut address = cpu.registers[self.rn as usize];

        if self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }

            if self.w {
                cpu.registers[self.rn as usize] = address;
            }
        }

        if self.l {
            let res = match mem.read_from_mem(address as usize) {
                Ok(n) => n,
                Err(e) => {
                    //TODO: Better error handling
                    println!("{}", e);
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

            cpu.registers[self.rd as usize] = value;
        } else {
            if self.s || !self.h{
                unreachable!();
            };
            // STRH
            match mem.write_halfword(address as usize, cpu.registers[self.rd as usize]) {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e);
                }
            }
        }

        if !self.p {
            if self.u {
                address += offset;
            } else {
                address -= offset;
            }
            cpu.registers[self.rn as usize] = address;
        }
    }
}

impl From<u32> for HalfwordDataOp {
    fn from(inst: u32) -> Self {
        let p = (inst >> 24 & 1) == 1;
        let byte_offset = ((inst & 0xff) | (inst >> 8 & 0xff)) as u8;
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
}
