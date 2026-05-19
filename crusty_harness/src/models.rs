use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Test {
    initial: CpuState,
    #[serde(rename="final")]
    end: CpuState,
    transactions: Vec<Transaction>,
    opcode: u32,
    base_addr: u32,
}

#[derive(Debug, Deserialize)]
pub struct CpuState {
    #[serde(rename="R")]
    registers: [u32; 16],
    #[serde(rename="R_fiq")]
    r_fiq: [u32; 7],
    #[serde(rename="R_svc")]
    r_svc: [u32; 2],
    #[serde(rename="R_abt")]
    r_abt: [u32; 2],
    #[serde(rename="R_irq")]
    r_irq: [u32; 2],
    #[serde(rename="R_und")]
    r_und: [u32; 2],
    #[serde(rename="CPSR")]
    cpsr: u32,
    #[serde(rename="SPSR")]
    spsr: [u32; 5],
    pipeline: [u32; 2],
    access: u32,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    kind: u32,
    size: u32,
    addr: u32,
    data: u32,
    cycle: u32,
    access: u32,
}
