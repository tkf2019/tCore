#![no_std]
#![allow(unused)]

extern crate alloc;

use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::String,
    vec::Vec,
};
use core::{fmt, slice::Iter};
use log::{debug, info, trace, warn};
use spin::{Lazy, Mutex};

pub mod testcases;

pub struct TestManger {
    pub cases: Option<&'static [&'static str]>,

    /// The number of passed tests
    pub passed: usize,

    /// Current test
    pub running: BTreeMap<String, usize>,

    /// A list of failed tests
    pub failed: Vec<String>,
}

impl TestManger {
    pub fn new() -> Self {
        Self {
            cases: None,
            passed: 0,
            running: BTreeMap::new(),
            failed: Vec::new(),
        }
    }

    /// Initialize the manager with target testcases.
    pub fn init(&mut self, cases: &'static [&'static str]) {
        self.cases = Some(cases);
    }

    /// Load a test.
    pub fn load(&mut self, name: &String) {
        self.running
            .entry(name.clone())
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    /// Update test result.
    pub fn exit(&mut self, exit_code: i32, name: &String) {
        if self.running.get(name).is_none() {
            return;
        }
        match exit_code {
            0 => {
                debug!("{} passed", name);
                self.passed += 1;
            }
            _ => {
                warn!("{} failed", name);
                self.failed.push(name.clone());
            }
        }
        self.running.entry(name.clone()).and_modify(|e| *e -= 1);
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
            let test_manager = TEST_MANAGER.lock();
            if test_manager.passed + test_manager.failed.len() == test_manager.cases.unwrap().len()
            {
                test_manager.info();
                panic!("TEST END");
            }
            None
        },
        |&user_command| {
            let argv = split_argv(user_command.as_bytes());
            TEST_MANAGER.lock().load(&argv[0]);
            Some(argv)
        },
    )
}

/// Finish the test with exit code.
pub fn finish_test(exit_code: i32, name: &String) {
    TEST_MANAGER.lock().exit(exit_code, name);
}
