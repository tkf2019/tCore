use riscv::register::time;
use time_subsys::{MSEC_PER_SEC, USEC_PER_SEC};

use crate::config::CLOCK_FREQ;

pub fn get_time() -> usize {
    time::read()
}

pub fn get_time_sec() -> usize {
    time::read() / CLOCK_FREQ
}

pub fn get_time_sec_f64() -> f64 {
    time::read() as f64 / CLOCK_FREQ as f64
}

pub fn get_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / USEC_PER_SEC)
}

#[inline]
pub fn set_timer(stime_value: u64) {
    sbi_rt::set_timer(stime_value);
}

