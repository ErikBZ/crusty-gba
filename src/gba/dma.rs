use crate::utils::Bitable;

pub(super) struct DmaControl {
    pub destination_control: u32,
    pub source_control: u32,
    pub dma_repeat: bool,
    pub dma_tfx_type: bool,
    pub gma_pak_drq: bool,
    pub dma_start_timing: u32,
    pub irq: bool,
    pub dma_enabled: bool,
}

impl From<u32> for DmaControl {
    fn from(value: u32) -> Self {
        DmaControl {
            destination_control: (value >> 5) & 0b11,
            source_control: (value >> 7) & 0b11,
            dma_repeat: value.bit_is_high(9),
            dma_tfx_type: value.bit_is_high(10),
            gma_pak_drq: value.bit_is_high(11),
            dma_start_timing: (value >> 12) & 0b11,
            irq: value.bit_is_high(14),
            dma_enabled: value.bit_is_high(15),
        }
    }
}
