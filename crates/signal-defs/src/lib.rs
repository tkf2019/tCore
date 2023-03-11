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

#[inline(always)]
pub const fn sigmask(sig: usize) -> u64 {
    1 << (sig as u64 - 1)
}

#[inline(always)]
pub const fn sigtest(sig: usize, mask: u64) -> bool {
    sigmask(sig) & mask != 0
}

#[inline(always)]
pub const fn sigvalid(sig: usize) -> bool {
    sig >= 1 && sig <= NSIG
}
pub const SIG_KERNEL_ONLY_MASK: u64 = sigmask(SIGKILL) | sigmask(SIGSTOP);

pub const SIG_KERNEL_STOP_MASK: u64 =
    sigmask(SIGSTOP) | sigmask(SIGTSTP) | sigmask(SIGTTIN) | sigmask(SIGTTOU);

pub const SIG_KERNEL_COREDUMP_MASK: u64 = sigmask(SIGQUIT)
    | sigmask(SIGILL)
    | sigmask(SIGTRAP)
    | sigmask(SIGABRT)
    | sigmask(SIGFPE)
    | sigmask(SIGSEGV)
    | sigmask(SIGBUS)
    | sigmask(SIGSYS)
    | sigmask(SIGXCPU)
    | sigmask(SIGXFSZ);

pub const SIG_KERNEL_IGNORE_MASK: u64 =
    sigmask(SIGCONT) | sigmask(SIGCHLD) | sigmask(SIGWINCH) | sigmask(SIGURG);

#[inline(always)]
pub fn sig_kernel_only(sig: usize) -> bool {
    sig == SIGKILL || sig == SIGSTOP
}

#[inline(always)]
pub fn sig_kernel_coredump(sig: usize) -> bool {
    sigtest(sig, SIG_KERNEL_COREDUMP_MASK)
}

#[inline(always)]
pub fn sig_kernel_ignore(sig: usize) -> bool {
    sigtest(sig, SIG_KERNEL_IGNORE_MASK)
}

#[inline(always)]
pub fn sig_kernel_stop(sig: usize) -> bool {
    sigtest(sig, SIG_KERNEL_STOP_MASK)
}
