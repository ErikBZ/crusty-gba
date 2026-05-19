use crate::gba::cpu::Cpu;
use crate::gba::debugger::{ContinueSubcommand, DebuggerCommand, MemoryBlock};
use crate::gba::system::SystemMemory;
use crate::ppu::Ppu;

use std::collections::HashSet;
use std::cmp::min;
use tracing::{event, Level};
use tracing_subscriber::{reload::Handle, Registry, filter::Targets};

pub fn run_debug(
    mut cpu: Cpu,
    mut memory: SystemMemory,
    mut ppu: Ppu,
    reload_handle: Handle<Targets, Registry>,
) {
    event!(Level::INFO, "Running Debug session");
    use std::io;
    let mut break_points: HashSet<usize> = HashSet::new();

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                continue;
            }
        }

        let cmd = match DebuggerCommand::parse(&input) {
            Ok(dc) => dc,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        match cmd {
            DebuggerCommand::BreakPoint(address) => {
                if break_points.contains(&address) {
                    break_points.remove(&address);
                } else {
                    break_points.insert(address);
                    println!("{:X?}", break_points);
                }
            }
            DebuggerCommand::Continue(ContinueSubcommand::Endless) => {
                cpu.tick(&mut memory);
                ppu.tick(cpu.cycles(), &mut memory);

                while !break_points.contains(&cpu.instruction_address()) {
                    cpu.tick(&mut memory);
                    if ppu.tick(cpu.cycles(), &mut memory) {
                        println!("{}", cpu);
                        let _ = ppu.get_next_frame(&memory);
                    }
                }
                println!("{}", cpu);
            }
            DebuggerCommand::Continue(ContinueSubcommand::For(l)) => {
                let mut n = 0;
                while !break_points.contains(&cpu.instruction_address()) && l > n {
                    cpu.tick(&mut memory);
                    if ppu.tick(cpu.cycles(), &mut memory) {
                        let _ = ppu.get_next_frame(&memory);
                        println!("{}", cpu);
                    };

                    n += 1;
                }
            }
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                if ppu.tick(cpu.cycles(), &mut memory) {
                    let _ = ppu.get_next_frame(&memory);
                }

                println!("{}", cpu);
            }
            DebuggerCommand::Info => {
                println!("{}", cpu);
            }
            DebuggerCommand::Quit => break,
            DebuggerCommand::LogLevel(lf) => {
                let _ = reload_handle.modify(|filter| {
                    *filter = Targets::default().with_target("crusty_gba", lf)
                });
            }
            DebuggerCommand::ReadMem(address) => match memory.read_word(address) {
                Ok(d) => println!("{:x}: {:x}", address, d),
                Err(e) => println!("{}", e),
            },
            DebuggerCommand::DumpMem(addr, block) => {
                let mem_slice = match memory.slice_map(addr) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("{}", e);
                        continue
                    }
                };

                let start_idx = (addr & 0xffffff) >> 2;
                if start_idx > mem_slice.len() {
                    println!("Address is out of range of memory block");
                    continue;
                }

                let range = match block {
                    MemoryBlock::Increase(b) => {
                        let end = min(start_idx + b / 4, mem_slice.len());
                        (start_idx..end).step_by(4)
                    },
                    MemoryBlock::Decrease(b) => {
                        let end = start_idx.saturating_sub(b / 4);
                        (start_idx..end).step_by(4)
                    },
                    MemoryBlock::ToEnd => (start_idx..mem_slice.len()).step_by(4),
                    MemoryBlock::ToStart => (0..start_idx).step_by(4)
                };

                for i in range {
                    println!(
                        "{:#010x}: {:#010x} {:#010x} {:#010x} {:#010x}",
                        i << 2,
                        mem_slice[i],
                        mem_slice[i+1],
                        mem_slice[i+2],
                        mem_slice[i+3],
                    );
                }
            }
            _ => (),
        }
    }
}
