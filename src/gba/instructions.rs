// TODO: For conditions codes look at page A3-6 of the ARM instruction manual
#[derive(Debug)]
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
    AL,
    NV
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

#[derive(Debug)]
pub enum Opcode {
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
    MVN
}

impl From<u32> for Opcode {
    fn from(inst: u32) -> Self {
        let code = (inst >> 20) & 0xf;
        match code {
            0 => Opcode::AND,
            1 => Opcode::EOR,
            2 => Opcode::SUB,
            3 => Opcode::RSB,
            4 => Opcode::ADD,
            5 => Opcode::ADC,
            6 => Opcode::SBC,
            7 => Opcode::RSC,
            8 => Opcode::TST,
            9 => Opcode::TEQ,
            10 => Opcode::CMP,
            11 => Opcode::CMN,
            12 => Opcode::ORR,
            13 => Opcode::MOV,
            14 => Opcode::BIC,
            _ => Opcode::MVN,
        }
    }
}

// TODO: Maybe rename this to DataOperation and use other structs
// like branch operation
pub struct CPUOperation {
    cond: Conditional,
    opcode: Opcode,
    s: bool,
    rn: u8,
    rd: u8,
    operand: u16
}

impl From<u32> for CPUOperation {
    fn from(inst: u32) -> Self {
        CPUOperation {
            cond: Conditional::from(inst),
            opcode: Opcode::from(inst),
            s: (inst >> 19 & 0x1) == 0x1,
            rd: (inst >> 11 & 0xf) as u8,
            rn: (inst >> 15 & 0xf) as u8,
            operand: (inst & 0xfff) as u16
        }
    }
}

impl CPUOperation {
    // TODO: impl Display instead and format the opcodes correctly, since some won't use all the
    // parts
    pub fn to_string(&self) -> String{
        format!("{:?} {:?} r{} r{} {}", self.opcode, self.cond, self.rn, self.rd, self.operand)
    }
}

pub fn is_data_processing(inst: u32) -> bool {
    inst & 0x0e000000 == 0x02000000
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
    inst & 0x0e000000 == 0x06000000
}

pub fn is_undefined(inst: u32) -> bool {
    inst & 0x0e000010 == 0x06000010
}

pub fn is_block_data_tfx(inst: u32) -> bool {
    inst & 0x0e000000 == 0x08000000
}

pub fn is_branch(inst: u32) -> bool {
    inst & 0x0e000000 == 0x09000000
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

