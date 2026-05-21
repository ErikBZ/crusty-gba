use std::collections::HashMap;

use serde::Serialize;

use crate::models::TestMemory;

/// Suite result is the sum of all tests in a single file
#[derive(Debug, Serialize)]
pub struct SuiteReport {
    failed_tests: HashMap<usize, TestError>,
    total: usize,
    failed: usize,
    passed: usize,
}

impl SuiteReport {
    pub fn new() -> Self {
        Self {
            failed_tests: HashMap::new(),
            total: 0,
            failed: 0,
            passed: 0
        }
    }

    pub fn add_success(&mut self) {
        self.total += 1;
        self.passed += 1;
    }

    pub fn add_failed(&mut self) {
        self.total += 1;
        self.failed += 1;
    }
}

#[derive(Debug, Serialize)]
struct TestError {
    opcode: u32,
    register: Option<HashMap<usize, Difference>>,
    fiq: Option<HashMap<usize, Difference>>,
    svc: Option<HashMap<usize, Difference>>,
    abt: Option<HashMap<usize, Difference>>,
    irq: Option<HashMap<usize, Difference>>,
    und: Option<HashMap<usize, Difference>>,
    cpsr: Option<Difference>,
    spsr: Option<Vec<Difference>>,
    optional: Option<TestMemory>
}

#[derive(Debug, Serialize)]
struct Difference {
    expected: u32,
    actual: u32
}
