#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::{fmt, slice::Iter};
use log::{info, trace};
use spin::{Lazy, Mutex};

pub mod testcases;

pub struct TestManger {
    pub cases: Option<&'static [&'static str]>,

    /// Current test (next to run)
    pub current: usize,

    /// The number of passed tests
    pub passed: usize,

    /// Current test
    pub running: BTreeSet<String>,

    /// A list of failed tests
    pub failed: Vec<String>,
}

impl TestManger {
    pub fn new() -> Self {
        Self {
            cases: None,
            passed: 0,
            current: 0,
            running: BTreeSet::new(),
            failed: Vec::new(),
        }
    }

    /// Initialize the manager with target testcases.
    pub fn init(&mut self, cases: &'static [&'static str]) {
        self.cases = Some(cases);
    }

    /// Load a test.
    pub fn load(&mut self, name: &String) {
        self.running.insert(name.clone());
    }

    /// Update test result.
    pub fn exit(&mut self, exit_code: i32, name: &String) {
        if !self.running.contains(name) {
            return;
        }
        match exit_code {
            0 => {
                self.passed += 1;
            }
            _ => {
                self.failed.push(name.clone());
            }
        }
        self.running.remove(name);
    }

    /// Show test status
    pub fn info(&self) {
        info!("Passed {} / {}", self.passed, self.cases.unwrap().len());
        info!("Failed {} tests:", self.failed.len());
        for test in &self.failed {
            info!("\t {}", test);
        }
    }
}

fn split_argv(s: &[u8]) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();
    let mut in_quotation = false;
    let mut start = 0;
    for pos in 0..s.len() {
        if s[pos] == '\"' as u8 {
            in_quotation = !in_quotation;
        } else if s[pos] == ' ' as u8 && !in_quotation {
            if pos > start {
                argv.push(core::str::from_utf8(&s[start..pos]).unwrap().into());
            }
            start = pos + 1;
        }
    }
    if start < s.len() {
        argv.push(core::str::from_utf8(&s[start..]).unwrap().into());
    }
    argv
}

static TEST_MANAGER: Lazy<Mutex<TestManger>> = Lazy::new(|| Mutex::new(TestManger::new()));
static TEST_ITER: Lazy<Mutex<core::slice::Iter<&str>>> =
    Lazy::new(|| Mutex::new(TEST_MANAGER.lock().cases.unwrap().into_iter()));

pub fn init(cases: &'static [&'static str]) {
    TEST_MANAGER.lock().init(cases);
}

/// Returns arguments of the test.
pub fn fetch_test() -> Option<Vec<String>> {
    TEST_ITER.lock().next().map_or_else(
        || {
            TEST_MANAGER.lock().info();
            None
        },
        |&user_command| {
            let argv = split_argv(user_command.as_bytes());
            TEST_MANAGER.lock().load(&user_command.into());
            Some(argv)
        },
    )
}

/// Finish the test with exit code.
pub fn finish_test(exit_code: i32, name: &String) {
    TEST_MANAGER.lock().exit(exit_code, name);
}
