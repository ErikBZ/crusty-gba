use tracing::warn;
use crate::{gba::system::MemoryError, SystemMemory};
// Base off of https://github.com/tuzz/game-loop 

// Values for controlling how the PPU draws pixels to the display
const DISP_CONTROL: usize = 0x4000000;
const DISP_STAT_ADDR: usize = 0x4000004;
const V_COUNT_ADDR: usize = 0x4000006;

const V_BLANK_FLAG: u32 = 0b00000001;
const H_BLANK_FLAG: u32 = 0b00000010;
const V_COUNTER_FLAG: u32 = 0b00000100;

fn set_flag_high(flag: u32, addr: usize, ram: &mut SystemMemory) -> Result<(), MemoryError> {
    let data = ram.read_halfword(addr)?;
    ram.write_halfword(addr, data | flag)
}

fn set_flag_low(flag: u32, addr: usize, ram: &mut SystemMemory) -> Result<(), MemoryError> {
    let data = ram.read_halfword(addr)?;
    ram.write_halfword(addr, data & !flag)
}

pub struct PPU {
    old_cycle: u32,
    frame_ready: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            old_cycle: 0,
            frame_ready: false,
        }
    }

    // A tick for the PPU is 4 cycles since that's how long it takes for 1 pixel to be drawn
    pub fn update(&mut self, cycle: u32, ram: &mut SystemMemory) {
        let new_cycle = cycle >> 2;
        if new_cycle == self.old_cycle {
            return;
        }
        let delta_cycle = new_cycle - self.old_cycle;

        for _ in 0..delta_cycle {
            self.internal_tick(ram);
        }

        self.old_cycle = new_cycle;
    }

    fn internal_tick(&mut self, ram: &mut SystemMemory) {
        match self.update_v_count(ram) {
            Ok(_) => (),
            Err(e) => warn!("{}", e)
        }
    }

    fn frame_done(&self) -> bool {
        self.old_cycle / 960 == 0
    }

    fn update_v_count(&self, ram: &mut SystemMemory) -> Result<(), MemoryError> {
        let mut data = ram.read_byte(V_COUNT_ADDR)?;
        let disp_stat_data = ram.read_byte(DISP_STAT_ADDR)?;

        if data >= 227 {
            data = 0;
            ram.write_byte(DISP_STAT_ADDR, disp_stat_data & !V_BLANK_FLAG)?;
        } else {
            data += 1;
        }

        if data > 160 {
            ram.write_byte(DISP_STAT_ADDR, disp_stat_data | V_BLANK_FLAG)?;
        }

        ram.write_byte(V_COUNT_ADDR, data)
    }

    pub fn generate_frame(&mut self) {
        self.frame_ready = false;
    }
} 
