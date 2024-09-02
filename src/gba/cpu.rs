pub struct CPU {
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
    r5: u32,
    r7: u32,
    // These registers aren't used with Thumb instructions
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    r12: u32,
    r13: u32,
    r14: u32,
    r15: u32,
    cpsr: u32,
    spsr0: u32,
    spsr1: u32,
    spsr2: u32,
    spsr3: u32,
    spsr4: u32,
}

impl CPU {
    // TODO: Make these mutable pointers?

    // Stack Pointer
    pub fn sp(&self) -> u32 {
        self.r13
    }

    // Link Register
    pub fn lr(&self) -> u32 {
        self.r14
    }

    // Program Counter
    pub fn pc(&self) -> u32 {
        self.r15
    }
}
