#![no_std]
#![allow(unused)]
#![feature(naked_functions)]

extern crate alloc;

mod instr;
mod register;
#[cfg(test)]
mod test;
