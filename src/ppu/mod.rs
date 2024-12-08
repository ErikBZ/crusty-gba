use tracing::warn;
use crate::{gba::system::MemoryError, SystemMemory};
// Base off of https://github.com/tuzz/game-loop 

const V_COUNT_ADDR: usize = 0x4000006;
const DISP_STAT_ADDR: usize = 0x4000004;

const V_BLANK_FLAG: u32 = 0b00000001;
const H_BLANK_FLAG: u32 = 0b00000010;
const V_COUNTER_FLAG: u32 = 0b00000100;

struct PPU {
    old_cycle: u32
}

impl PPU {
    pub fn tick(&mut self, cycle: u32, ram: &mut SystemMemory) {
        if cycle >> 2 == self.old_cycle {
            return;
        }
        self.old_cycle = cycle >> 2;

        self.update_v_count(ram);
    }

    fn frame_done(&self) -> bool {
        self.old_cycle / 960 == 0
    }

    fn update_v_count(&self, ram: &mut SystemMemory) -> Result<(), MemoryError> {
        let mut data = ram.read_byte(V_COUNT_ADDR)?;

        if data >= 227 {
            data = 0;
        } else {
            data += 1;
        }

        if data > 160 {

        }

        ram.write_byte(V_COUNT_ADDR, data)
    }

    pub fn render() {
        todo!()
    }
} 
