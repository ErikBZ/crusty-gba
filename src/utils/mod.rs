pub mod shifter;
use shifter::ShiftWithCarry;

// Returns true if the bit at x is 1
pub fn bit_is_one_at(num: u32, x: u32) -> bool {
    if x > 31 {
        panic!()
    }

    (num >> x) & 1 == 1
}

pub trait Bitable {
    fn bit_is_high(&self, x: u32) -> bool;
}

impl Bitable for u64 {
    fn bit_is_high(&self, x: u32) -> bool {
        // in release this should be false
        if x > 63 {
            panic!()
        } else {
            (self >> x) & 1 == 1
        }
    }
}

impl Bitable for u32 {
    fn bit_is_high(&self, x: u32) -> bool {
        // in release this should be false
        if x > 31 {
            panic!()
        } else {
            (self >> x) & 1 == 1
        }
    }
}
