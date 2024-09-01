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

