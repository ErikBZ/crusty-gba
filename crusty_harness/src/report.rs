use std::collections::HashMap;
use std::iter::zip;

use crusty::Cpu;
use serde::Serialize;
use tracing::trace;

use crate::models::TestMemory;

/// Suite result is the sum of all tests in a single file
#[derive(Debug, Serialize)]
pub struct SuiteReport {
    path: String,
    failed_tests: HashMap<usize, TestError>,
    pub total: usize,
    pub failed: usize,
    pub passed: usize,
    pub skipped: usize,
}

impl SuiteReport {
    pub fn new(path: String) -> Self {
        Self {
            path,
            failed_tests: HashMap::new(),
            total: 0,
            failed: 0,
            passed: 0,
            skipped: 0
        }
    }

    pub fn add_success(&mut self) {
        self.total += 1;
        self.passed += 1;
    }

    pub fn add_skipped(&mut self) {
        self.skipped += 1;
    }

    pub fn add_failed(&mut self, idx: usize, e: TestError) {
        self.failed_tests.insert(idx, e);
        self.total += 1;
        self.failed += 1;
    }
}

// NOTE: Wow all these skip_serializing_if do look ugly
#[derive(Debug, Serialize)]
pub struct TestError {
    opcode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    instruction_address: Option<Difference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    register: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fiq: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    svc: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    abt: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    irq: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    und: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpsr: Option<Difference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spsr: Option<HashMap<usize, Difference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mem: Option<TestMemory>
}

impl TestError {
    pub fn new(opcode: u32) -> Self {
        let opcode = format!("{:#010x}", opcode);
        Self {
            opcode,
            instruction_address: None,
            register: None,
            fiq: None,
            svc: None,
            abt: None,
            irq: None,
            und: None,
            cpsr: None,
            spsr: None,
            mem: None
        }
    }

    pub fn apply_differences(mut self, expected: Cpu, actual: Cpu) -> Self {
        if expected.cpsr != actual.cpsr {
            self.add_cpsr_difference(Difference { actual: expected.cpsr, expected: actual.cpsr });
        }

        if expected.instruction_address() != actual.instruction_address() {
            self.add_cpsr_difference(Difference { actual: expected.inst_addr as u32, expected: actual.inst_addr as u32 });
        }

        for (idx, (a, e)) in zip(actual.registers, expected.registers).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_register_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.fiq_banked_gen_regs, expected.fiq_banked_gen_regs).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_fiq_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.svc_banked_regs, expected.svc_banked_regs).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_svc_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.abt_banked_regs, expected.abt_banked_regs).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_abt_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.irq_banked_regs, expected.irq_banked_regs).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_irq_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.und_banked_regs, expected.und_banked_regs).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_und_difference(idx, diff);
            }
        }

        for (idx, (a, e)) in zip(actual.psr, expected.psr).enumerate() {
            if a != e {
                let diff = Difference::new(a, e);
                self.add_spsr_difference(idx, diff);
            }
        }

        self
    }

    /// This overwrites whatever existed before
    pub fn add_cpsr_difference(&mut self, diff: Difference) {
        self.cpsr = Some(diff);
    }

    // NOTE: Could be a a macro but idk how to do it
    pub fn add_register_difference(&mut self, idx: usize, diff: Difference) {
        if self.register.is_none() {
            self.register = Some(HashMap::new());
        }

        let m = self.register.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_fiq_difference(&mut self, idx: usize, diff: Difference) {
        if self.fiq.is_none() {
            self.fiq = Some(HashMap::new());
        }

        let m = self.fiq.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_svc_difference(&mut self, idx: usize, diff: Difference) {
        if self.svc.is_none() {
            self.svc = Some(HashMap::new());
        }

        let m = self.svc.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_abt_difference(&mut self, idx: usize, diff: Difference) {
        if self.abt.is_none() {
            self.abt = Some(HashMap::new());
        }

        let m = self.abt.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_irq_difference(&mut self, idx: usize, diff: Difference) {
        if self.irq.is_none() {
            self.irq = Some(HashMap::new());
        }

        let m = self.irq.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_und_difference(&mut self, idx: usize, diff: Difference) {
        if self.und.is_none() {
            self.und = Some(HashMap::new());
        }

        let m = self.und.as_mut().unwrap();
        m.insert(idx, diff);
    }

    pub fn add_spsr_difference(&mut self, idx: usize, diff: Difference) {
        if self.spsr.is_none() {
            self.spsr = Some(HashMap::new());
        }

        let m = self.spsr.as_mut().unwrap();
        m.insert(idx, diff);
    }
}

#[derive(Debug, Serialize)]
pub struct Difference {
    actual: u32,
    expected: u32,
}

impl Difference {
    pub fn new(expected: u32, actual: u32) -> Self {
        Self { actual, expected }
    }
}
