pub mod shifter;

// Returns true if the bit at x is 1
pub fn bit_is_one_at(num: u32, x: u32) -> bool {
    if x > 31 {
        panic!()
    }

    (num >> x) & 1 == 1
}

pub trait Bitable {
    fn bit_is_high(&self, x: u32) -> bool;
    fn half_byte_at(&self, x: u32) -> u32;
    fn byte_at(&self, x: u32) -> u32;
    fn halfword_at(&self, x: u32) -> u32;
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

    fn half_byte_at(&self, x: u32) -> u32 {
        if x > 63 {
            panic!()
        } else {
            ((self >> x) & 0xf) as u32
        }
    }

    fn byte_at(&self, _x: u32) -> u32 {
        todo!()
    }

    fn halfword_at(&self, _x: u32) -> u32 {
        todo!()
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

    fn half_byte_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xf
    }

    fn byte_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xff
    }

    fn halfword_at(&self, x: u32) -> u32 {
        guard_shift(*self, x) & 0xffff
    }
}

fn guard_shift(val: u32, x: u32) -> u32 {
    if x > 31 {
        panic!()
    }
    val >> x
}
