#![no_std]
#![allow(unused)]
#![allow(non_upper_case_globals)]
#![feature(sync_unsafe_cell)]
#![feature(negative_impls)]

extern crate alloc;

mod arch;
mod rcu;
mod rwlock;
mod seqlock;
mod sleeplock;
mod spinlock;

pub use rcu::{reclamation, wait, RcuCell, RcuDrop, RcuDropFn, RcuReadGuard, RcuType};
pub use seqlock::SeqLock;
pub use sleeplock::{Sched as SleepLockSched, SleepLock, SleepLockGuard};
pub use spinlock::{SpinLock, SpinLockGuard};

use arch::*;

const NCPU: usize = 16;

/// Per-CPU state
#[derive(Debug, Default, Clone, Copy)]
pub struct CPU {
    /// Depth of push_off() nesting.
    pub noff: usize,

    /// Were interrupts enabled before push_off()?
    pub intena: bool,
}

pub static mut CPUs: [CPU; NCPU] = [CPU {
    noff: 0,
    intena: false,
}; NCPU];

/// Save old interrupt enabling bit in CPU local variables and disable interrupt at first
/// `push_off()`. The depth of nesting is increased by 1.
#[inline(always)]
pub fn push_off() {
    #[cfg(target_os = "none")]
    {
        let old = intr_get();
        intr_off();
        let cpu = unsafe { &mut CPUs[cpu_id()] };
        if cpu.noff == 0 {
            cpu.intena = old;
        }
        cpu.noff += 1;
    }
}

/// Restore old interrupt enabling bit in CPU local variables and enable interrupt at the last
/// `pop_off()`. The depth of nesting is decreased by 1.
#[inline(always)]
pub fn pop_off() {
    #[cfg(target_os = "none")]
    {
        let cpu = unsafe { &mut CPUs[cpu_id()] };

        assert!(!intr_get() && cpu.noff >= 1);

        cpu.noff -= 1;
        if cpu.noff == 0 && cpu.intena {
            intr_on();
        }
    }
}
