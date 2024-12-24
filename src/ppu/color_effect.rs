use crate::utils::Bitable;

pub(super) struct InternalColorEffect {
    pub bg0: bool,
    pub bg1: bool,
    pub bg2: bool,
    pub bg3: bool,
    pub obj: bool,
    pub bd: bool,
}

impl From<u32> for InternalColorEffect {
    fn from(value: u32) -> Self {
        InternalColorEffect {
            bg0: value.bit_is_high(0),
            bg1: value.bit_is_high(1),
            bg2: value.bit_is_high(2),
            bg3: value.bit_is_high(3),
            obj: value.bit_is_high(4),
            bd: value.bit_is_high(5),
        }
    }
}

pub(super) enum ColorEffect {
    AlphaBlending{ eva: EffectCoef, evb: EffectCoef},
    BrightnessIncrease(EffectCoef),
    BrightnessDecrease(EffectCoef)
}

// EffectCoef maxes out at 16
pub(super) struct EffectCoef(u32);

impl From<u32> for EffectCoef {
    fn from(value: u32) -> Self {
        let x = value & 0b1111;
        if x > 16 {
            EffectCoef(16)
        } else {
            EffectCoef(x)
        }
    }
}

pub(super) struct ColorEffectSelection {
    pub first_target: InternalColorEffect,
    pub second_target: InternalColorEffect,
    pub effect: Option<ColorEffect>
}

impl From<(u32, u32)> for ColorEffectSelection {
    fn from(value: (u32, u32)) -> Self {
        let effect = match (value.0 >> 6) & 0b11 {
            0 => None,
            1 => Some(ColorEffect::AlphaBlending { 
                eva: EffectCoef::from(value.0 >> 15),
                evb: EffectCoef::from(value.0 >> 23)
            }),
            2 => Some(ColorEffect::BrightnessIncrease (
                EffectCoef(value.1)
            )),
            3 => Some(ColorEffect::BrightnessDecrease (
                EffectCoef(value.1)
            )),
            _ => panic!()
        };
        ColorEffectSelection {
            first_target: InternalColorEffect::from(value.0 & 0x1f),
            second_target: InternalColorEffect::from(value.0 >> 8),
            effect
        }
    }
}
