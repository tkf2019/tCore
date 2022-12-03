//! The information is passed on to the user processes by binary loaders which
//! are part of the kernel subsystem itself; either built-in the kernel or a
//! kernel module.
//!
//! See `http://articles.manugarg.com/aboutelfauxiliaryvectors.html`.

use core::{
    mem::{align_of, size_of},
    ops::{Deref, DerefMut},
    ptr::null,
};

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use log::trace;
use tmm_rv::{PhysAddr, VirtAddr};

use super::flags::AuxType;

pub struct InitInfo {
    /// Argument strings
    pub args: Vec<String>,

    /// Environment strings
    pub envs: Vec<String>,

    /// Auxiliary value
    pub auxv: BTreeMap<AuxType, usize>,
}

pub struct InitStack {
    /// Stack pointer (low physical address).
    pub sp: PhysAddr,

    /// Stack base (high fixed physical address).
    pub base: PhysAddr,

    /// Stack pointer (low virtual address).
    pub vsp: VirtAddr,

    /// Stack base (high fixed virtual address).
    pub vbase: VirtAddr,
}

impl InitStack {
    pub fn new(sp: PhysAddr, vsp: VirtAddr) -> Self {
        Self {
            sp,
            base: sp,
            vsp,
            vbase: vsp,
        }
    }

    /// Pushes a slice on the stack.
    ///
    /// This function may cause kernel page fault due to wrong physical address of `sp`.
    /// Carefully configure the page table before trying to call it.
    ///
    /// Returns current stack pointer (virtual address).
    pub fn push_slice<T: Copy>(&mut self, v: &[T]) -> VirtAddr {
        self.sp -= v.len() * size_of::<T>();
        self.sp -= self.sp.value() % align_of::<T>();
        unsafe { core::slice::from_raw_parts_mut(self.sp.value() as *mut T, v.len()) }
            .copy_from_slice(v);

        self.vsp -= v.len() * size_of::<T>();
        self.vsp -= self.vsp.value() % align_of::<T>();
        self.vsp
    }

    /// Pushes a string on the stack.
    ///
    /// Returns current stack pointer (virtual address).
    pub fn push_str(&mut self, s: &str) -> VirtAddr {
        self.push_slice(&[b'\0']);
        self.push_slice(s.as_bytes());
        self.vsp
    }

    /// Serialized args, envp, auxv.
    pub fn serialize(v: InitInfo, sp: PhysAddr, vsp: VirtAddr) -> Self {
        let mut stack = InitStack::new(sp, vsp);
        stack.push_str(&v.args[0]);
        // random string: 16 bytes
        let random = stack.push_slice(&[0usize, 0usize]);
        // environment strings
        let envs: Vec<VirtAddr> = v
            .envs
            .iter()
            .map(|env| stack.push_str(env.as_str()))
            .collect();
        // argv strings
        let argv: Vec<VirtAddr> = v
            .args
            .iter()
            .map(|arg| stack.push_str(arg.as_str()))
            .collect();
        // padding: 16 bytes
        stack.push_slice(&[null::<u8>(), null::<u8>()]);
        // ELF Auxiliary Table
        for (&type_, &value) in v.auxv.iter() {
            match type_ {
                AuxType::RANDOM => stack.push_slice(&[type_.into(), random.value()]),
                _ => stack.push_slice(&[type_.into(), value]),
            };
        }
        // NULL that ends envp[]
        stack.push_slice(&[null::<u8>()]);
        // env pointers
        stack.push_slice(envs.as_slice());
        // NULL that ends argv[]
        stack.push_slice(&[null::<u8>()]);
        // argv pointers
        stack.push_slice(argv.as_slice());
        // argc
        stack.push_slice(&[argv.len()]);
        stack
    }
}

impl Deref for InitStack {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            core::slice::from_raw_parts(self.sp.value() as *const _, (self.base - self.sp).into())
        }
    }
}

impl DerefMut for InitStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            core::slice::from_raw_parts_mut(self.sp.value() as *mut _, (self.base - self.sp).into())
        }
    }
}
