#![no_std]
#![feature(linked_list_remove)]

extern crate alloc;

mod sigaction;
mod siginfo;
mod signo;
mod sigpending;
mod sigset;

pub use sigaction::*;
pub use siginfo::*;
pub use signo::*;
pub use sigpending::*;
pub use sigset::*;

pub fn sigmask(signo: SignalNo) -> u64 {
    1 << (usize::from(signo) as u64 - 1)
}