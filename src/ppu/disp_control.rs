use crate::{gba::system::{MemoryError, SystemMemory}, utils::{bit_is_one_at, Bitable}};
const DISP_CONTROL: usize = 0x4000000;
const DISP_STAT: usize = 0x4000004;

pub fn display_control(ram: &mut SystemMemory) -> Result<DisplayControl, MemoryError> {
    let data = ram.read_word(DISP_CONTROL)?;
    Ok(DisplayControl::from(data))
}

pub fn display_stat(ram: &mut SystemMemory) -> Result<DisplayStat, MemoryError> {
    let data = ram.read_word(DISP_STAT)?;
    Ok(DisplayStat::from(data))
}

#[derive(Debug)]
pub(super) struct DisplayControl {
    pub bg_mode: u32,
    pub gbc_mode: bool,
    pub display_frame_select: bool,
    pub h_blank_interval_free: bool,
    pub obj_character_mapping: bool,
    pub forced_blank: bool,
    pub display_bg0: bool,
    pub display_bg1: bool,
    pub display_bg2: bool,
    pub display_bg3: bool,
    pub display_obj: bool,
    pub display_window0:bool,
    pub display_window1:bool,
    pub display_window_obj: bool
}

impl From<u32> for DisplayControl {
    fn from(value: u32) -> Self {
        DisplayControl {
            bg_mode: value & 0b111,
            gbc_mode: value.bit_is_high(3),
            display_frame_select: value.bit_is_high(4),
            h_blank_interval_free: value.bit_is_high(5),
            obj_character_mapping: value.bit_is_high(6),
            forced_blank: value.bit_is_high(7),
            display_bg0: value.bit_is_high(8),
            display_bg1: value.bit_is_high(9),
            display_bg2: value.bit_is_high(10),
            display_bg3: value.bit_is_high(11),
            display_obj: value.bit_is_high(12),
            display_window0: value.bit_is_high(13),
            display_window1: value.bit_is_high(14),
            display_window_obj: value.bit_is_high(15),
        }
    }
}

pub(super) struct DisplayStat {
    pub v_blank: bool,
    pub h_blank: bool,
    pub v_counter: bool,
    pub v_blank_irq: bool,
    pub h_blank_irq: bool,
    pub v_counter_irq: bool,
    pub v_count_setting: u32,
    pub v_count: u32,
}

impl From<u32> for DisplayStat {
    fn from(value: u32) -> Self {
        DisplayStat {
            v_blank: value.bit_is_high(0),
            h_blank: value.bit_is_high(1),
            v_counter: value.bit_is_high(2),
            v_blank_irq: value.bit_is_high(3),
            h_blank_irq: value.bit_is_high(4),
            v_counter_irq: value.bit_is_high(5),
            v_count_setting: value.byte_at(8),
            v_count: value.byte_at(16),
        }
    }
}
