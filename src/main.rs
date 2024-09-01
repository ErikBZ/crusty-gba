mod gba;
use std::fs::File;
use std::io::prelude::*;

use gba::instructions::{CPUOperation, Conditional, Opcode};

fn main() {
    let mut file = match File::open("test.gba") {
        Ok(f) => f,
        Err(e) => {
            println!("There was an error: {:?}", e);
            return;
        }
    };
    let codes: Vec<u32> = read_file_into_u32(&mut file);
    println!("Size of file: {:?}", codes.len()*4);

    for i in 0..100 {
        let cond = Conditional::from(codes[i]);
        // TODO: Pretty sure some of these checks aren't correct
        if codes[i] & 0x0e000000 == 0x0a000000 {
            let offest = codes[i] & 0x00ffffff;
            let b_type = if 0x01000000 & codes[i] == 0 {
                "B"
            } else {
                "BL"
            };
            println!("{:?} {} +{:#x}", cond, b_type, offest);
        } else if codes[i] & 0x0ffffff0 == 0x012fff10 {
            let offest = codes[i] & 0x00ffffff;
            println!("{:?} BX +{:#x}", cond, offest);
        } else if codes[i] & 0x06000010 == 0x06000010 {
            println!("undefined");
        } else if codes[i] & 0x0e000000 == 0x02000000 {
            let op = CPUOperation::from(codes[i]);
            println!("{}", op.to_string());
        } else {
            println!("{:032b}, 0x{:08x}, {:?}", codes[i], codes[i], cond);
        }

    }
}

fn read_file_into_u32(file: &mut File) -> Vec<u32> {
    let mut instructions = Vec::new();  

    loop {
        let mut buffer = [0; 4];

        let n = match file.take(4).read(&mut buffer) {
            Ok(n) => n,
            Err(_) => break
        };

        if n == 0 { break; }
        instructions.push(u32::from_le_bytes(buffer));
        if n < 4 { break; }
    }

    instructions
}

