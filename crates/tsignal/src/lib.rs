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

use terrno::Errno;

pub trait SignalCore {
    fn send_signal(sig: usize, info: &SigInfo, pendings: &mut SigPending) -> Result<(), Errno>;
}
