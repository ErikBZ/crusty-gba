mod gba;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use gba::Conditional;
use gba::cpu::CPU;
use gba::debugger::DebuggerCommand;
use gba::system::SystemMemory;
use gba::arm::decode_as_arm;
use gba::thumb::decode_as_thumb;

fn main() {
    let num = 0xbc2 - 0xab8;
    println!("{:x} {}", num, num);
    // TODO: Just put test.gba in the root dir
    let mut file = match File::open("test.gba") {
        Ok(f) => f,
        Err(e) => {
            println!("There was an error: {:?}", e);
            return;
        }
    };
    let codes: Vec<u32> = read_file_into_u32(&mut file);
    debug_bios(codes);
}

fn debug_bios(codes: Vec<u32>) {
    use std::io;
    let mut cpu = CPU::default();
    let mut memory =  SystemMemory::default();
    memory.copy_bios(codes);
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
            DebuggerCommand::Continue => {
                while !break_points.contains(&cpu.pc()) {
                    println!("{:x}", cpu.pc());
                    cpu.tick(&mut memory);
                }
            },
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                let op = if !cpu.is_thumb_mode() {
                    decode_as_arm(cpu.decode)
                } else {
                    decode_as_thumb(cpu.decode)
                };
                let cond = Conditional::from(cpu.decode);

                println!("{}", cpu);
                if cpu.is_thumb_mode() {
                    println!("{:#04x} {:?} {:?}", cpu.decode, cond, op);
                } else {
                    println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
                }
            },
            DebuggerCommand::Info => {
                let op = if !cpu.is_thumb_mode() {
                    decode_as_arm(cpu.decode)
                } else {
                    decode_as_thumb(cpu.decode)
                };
                let cond = Conditional::from(cpu.decode);

                println!("{}", cpu);
                if cpu.is_thumb_mode() {
                    println!("{:#04x} {:?} {:?}", cpu.decode, cond, op);
                } else {
                    println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
                }            },
            DebuggerCommand::Quit => break,
        }
    }
}

fn read_file_into_u32(file: &mut File) -> Vec<u32> {
    let mut instructions = Vec::new();

    loop {
        let mut buffer = [0; 4];

        let n = match file.take(4).read(&mut buffer) {
            Ok(n) => n,
            Err(_) => break,
        };

        if n == 0 {
            break;
        }
        instructions.push(u32::from_le_bytes(buffer));
        if n < 4 {
            break;
        }
    }

    instructions
}
