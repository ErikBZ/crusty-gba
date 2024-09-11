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

    pub fn run_instruction(&self, inst: u32) {
    }
}
