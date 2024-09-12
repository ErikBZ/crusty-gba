use super::arm::ArmInstruction;

#[derive(Default, Debug)]
pub struct CPU {
    registers: [u32; 16],
    cpsr: u32,
    thumb: bool
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
        self.registers[15]
    }
    pub fn set_pc(&mut self, pc: u32) {
        self.registers[15] = pc;
    }

    pub fn run_instruction(&self, inst: u32, ram: &mut [u32; 128]) {
        let op = ArmInstruction::from(inst);

        match op {
            ArmInstruction::CMP(cond, o) =>  {

            }
            _ => {},
        }
    }
}
