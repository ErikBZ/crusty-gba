use crate::{gba::system::{SystemMemory, MemoryError}, utils::Bitable};
use crate::utils::io_registers::{INTERRUPT_ENABLE, INTERRUPT_REQUEST, INTERRUPT_MASTER_ENABLE};

pub struct InterruptMasterEnable (bool);

pub fn interrupt_enable(ram: &SystemMemory) -> Result<InterruptEnableOrRequest, MemoryError> {
    let data = ram.read_halfword(INTERRUPT_ENABLE)?;
    Ok(InterruptEnableOrRequest::from(data))
}

pub fn interrupt_request(ram: &SystemMemory) -> Result<InterruptEnableOrRequest,MemoryError> {
    let data = ram.read_halfword(INTERRUPT_REQUEST)?;
    Ok(InterruptEnableOrRequest::from(data))
}

pub fn interrupt_master_enable(ram: &SystemMemory) -> Result<InterruptMasterEnable,MemoryError> {
    let data = ram.read_halfword(INTERRUPT_MASTER_ENABLE)?;
    Ok(InterruptMasterEnable::from(data))
}

impl From<u32> for InterruptMasterEnable {
    fn from(value: u32) -> Self {
        Self(value.bit_is_high(0))
    }
}

// Note: Just use this for both the enable, an request map
pub struct InterruptEnableOrRequest {
    pub lcd_v_blank: bool,
    pub lcd_h_blank: bool,
    pub lcd_v_counter_match: bool,
    pub timer_0_overflow: bool,
    pub timer_1_overflow: bool,
    pub timer_2_overflow: bool,
    pub timer_3_overflow: bool,
    pub serial_communication: bool,
    pub dma_0: bool,
    pub dma_1: bool,
    pub dma_2: bool,
    pub dma_3: bool,
    pub keypad: bool,
    pub game_pak: bool,
}

impl From<u32> for InterruptEnableOrRequest {
    fn from(value: u32) -> Self {
        Self {
            lcd_v_blank: value.bit_is_high(0),
            lcd_h_blank: value.bit_is_high(1),
            lcd_v_counter_match: value.bit_is_high(2),
            timer_0_overflow: value.bit_is_high(3),
            timer_1_overflow: value.bit_is_high(4),
            timer_2_overflow: value.bit_is_high(5),
            timer_3_overflow: value.bit_is_high(6),
            serial_communication: value.bit_is_high(7),
            dma_0: value.bit_is_high(8),
            dma_1: value.bit_is_high(9),
            dma_2: value.bit_is_high(10),
            dma_3: value.bit_is_high(11),
            keypad: value.bit_is_high(12),
            game_pak: value.bit_is_high(13),
        }
    }
}

