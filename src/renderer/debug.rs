use std::collections::HashSet;
use crate::gba::cpu::Cpu;
use crate::ppu::Ppu;
use crate::gba::debugger::{DebuggerCommand, ContinueSubcommand};
use crate::gba::system::SystemMemory;
use tracing::{event, Level};
use tracing_subscriber::{reload::Handle, Registry};
use tracing_subscriber::filter::LevelFilter;

pub fn run_debug(mut cpu: Cpu, mut memory: SystemMemory, mut ppu: Ppu, reload_handle: Handle<LevelFilter, Registry>) {
    event!(Level::INFO, "Running Debug session");
    use std::io;
    let mut break_points: HashSet<usize> = HashSet::new();

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {},
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
            },
        };

        match cmd {
            DebuggerCommand::BreakPoint(address) => {
                if break_points.contains(&address) {
                    break_points.remove(&address);
                } else {
                    break_points.insert(address);
                    println!("{:?}", break_points);
                }
            },
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
            },
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
            },
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                if ppu.tick(cpu.cycles(), &mut memory) {
                    let _ = ppu.get_next_frame(&memory);
                }

                println!("{}", cpu);
            },
            DebuggerCommand::Info => {
                println!("{}", cpu);
            },
            DebuggerCommand::Quit => break,
            DebuggerCommand::LogLevel(lf) => {
                let _ = reload_handle.modify(|filter| *filter = lf);
            },
            DebuggerCommand::ReadMem(address) => {
                match memory.read_word(address) {
                    Ok(d) =>  println!("{:x}: {:x}", address, d),
                    Err(e) => println!("{}", e),
                }
            }
            _ => (),
        }
    }
}

