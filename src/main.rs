mod gba;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

extern crate strum_macros;

use gba::Conditional;
use gba::thumb::ThumbInstruction;
use gba::cpu::CPU;
use gba::debugger::DebuggerCommand;
use gba::system::SystemMemory;
use gba::arm::decode_as_arm;

fn main() {
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
    let mut break_points: HashSet<u32> = HashSet::new();

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
                for _ in 0..100 {
                    println!("{:x}", cpu.pc());
                    if break_points.contains(&cpu.pc()) {
                        break;
                    }
                    cpu.tick(&mut memory);
                }
            },
            DebuggerCommand::Next => {
                cpu.tick(&mut memory);
                let op = decode_as_arm(cpu.decode);
                let cond = Conditional::from(cpu.decode);
                println!("{}", cpu);
                println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
            },
            DebuggerCommand::Info => {
                let op = if !cpu.is_thumb_mode() {
                    decode_as_arm(cpu.decode)
                } else {
                    println!("Address is not within ROM");
                    continue;
                };
                let cond = Conditional::from(cpu.decode);
                println!("{}", cpu);
                println!("{:#08x} {:?} {:?}", cpu.decode, cond, op);
            },
            DebuggerCommand::Quit => break,
        }
    }
}

fn dump_opcodes(num_of_lines: usize, codes: Vec<u32>) -> Result<(), ()> {
    if num_of_lines < codes.len() {
        return Err(())
    };
    let mut decode_thumb = false;

    for i in 0..0x100 {
        let inst_address = i << 2;
        if !decode_thumb {
            let op = decode_as_arm(codes[i]);
            println!("{:#08x} {:0x} {:?}", inst_address, codes[i], op);
        } else {
            let op2 = ThumbInstruction::from((codes[i] >> 16) as u16);
            let op1 = ThumbInstruction::from((codes[i] & 0xffff) as u16);
            match op1 {
                ThumbInstruction::Undefined => decode_thumb = false,
                _ => println!("{:#08x} {:0x} {:?}", inst_address, codes[i], op1),
            };
            match op2 {
                ThumbInstruction::Undefined => decode_thumb = false,
                _ => println!("{:#08x} {:0x} {:?}", inst_address+2, codes[i], op2),
            }
        }
    }
    Ok(())
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
