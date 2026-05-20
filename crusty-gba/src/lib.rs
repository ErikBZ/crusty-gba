pub mod cli;
pub mod gba;
pub mod ppu;
pub mod renderer;
pub mod utils;
pub mod memory;

// TODO: Get rid of this since it messes with the use's in the submodules
pub use crate::gba::system::SystemMemory;
pub use gba::cpu::Cpu;
