use std::fmt::{Display, Formatter, Result};
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionDecodeError {
    ConditionalNotValid { value: u32, cond: u32 },
    NoMatchingOperation(u32),
}

impl Display for InstructionDecodeError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::ConditionalNotValid { value, cond } => {
                write!(f, "Condition {cond} not valid for instruction: {value}")
            }
            Self::NoMatchingOperation(v) => write!(f, "No matching operation for instruction: {v}"),
        }
    }
}
