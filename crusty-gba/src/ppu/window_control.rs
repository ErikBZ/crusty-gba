use crate::utils::Bitable;

pub(super) struct WindowDimensions {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

// For Window 1 we need to shift these boths 16 to the right
impl From<(u32, u32)> for WindowDimensions {
    fn from(value: (u32, u32)) -> Self {
        WindowDimensions {
            left: value.0.byte_at(8),
            right: value.0.byte_at(0),
            bottom: value.1.byte_at(0),
            top: value.1.byte_at(8),
        }
    }
}

pub(super) struct InternalWindowCnt {
    pub bg0: bool,
    pub bg1: bool,
    pub bg2: bool,
    pub bg3: bool,
    pub obj: bool,
    pub color_special: bool,
}

impl From<u32> for InternalWindowCnt {
    fn from(value: u32) -> Self {
        InternalWindowCnt {
            bg0: value.bit_is_high(0),
            bg1: value.bit_is_high(1),
            bg2: value.bit_is_high(2),
            bg3: value.bit_is_high(3),
            obj: value.bit_is_high(4),
            color_special: value.bit_is_high(5),
        }
    }
}

pub(super) struct WindowCnt {
    pub window_0: InternalWindowCnt,
    pub window_1: InternalWindowCnt,
    pub outside: InternalWindowCnt,
    pub obj: InternalWindowCnt,
}

impl From<u32> for WindowCnt {
    fn from(value: u32) -> Self {
        WindowCnt {
            window_0: InternalWindowCnt::from(value),
            window_1: InternalWindowCnt::from(value >> 8),
            outside: InternalWindowCnt::from(value >> 16),
            obj: InternalWindowCnt::from(value >> 24),
        }
    }
}
