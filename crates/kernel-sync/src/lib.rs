#![no_std]
#![allow(unused)]
#![allow(non_upper_case_globals)]

extern crate alloc;

mod arch;
mod rcu;
mod sleep;
mod spin;

pub use crate::spin::{SpinMutex as Mutex, SpinMutexGuard as MutexGuard};
use arch::*;
pub use sleep::{Sched, SleepMutex, SleepMutexGuard};

const NCPU: usize = 16;

/// Per-CPU state
#[derive(Debug, Default, Clone, Copy)]
pub struct CPU {
    /// Depth of push_off() nesting.
    noff: usize,

    /// Were interrupts enabled before push_off()?
    intena: bool,
}

static mut CPUs: [CPU; NCPU] = [CPU {
    noff: 0,
    intena: false,
}; NCPU];

/// Save old interrupt enabling bit in CPU local variables and disable interrupt at first
/// `push_off()`. The depth of nesting is increased by 1.
pub fn push_off() {
    let old = intr_get();
    intr_off();
    let cpu = unsafe { &mut CPUs[cpu_id()] };
    if cpu.noff == 0 {
        cpu.intena = old;
    }
    cpu.noff += 1;
}

/// Restore old interrupt enabling bit in CPU local variables and enable interrupt at the last
/// `pop_off()`. The depth of nesting is decreased by 1.
pub fn pop_off() {
    let cpu = unsafe { &mut CPUs[cpu_id()] };

    assert!(!intr_get() && cpu.noff >= 1);

    cpu.noff -= 1;
    if cpu.noff == 0 && cpu.intena {
        intr_on();
    }
}
