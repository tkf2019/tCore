mod logger;
mod panic;

use core::fmt::{Arguments, Result, Write};
pub use logger::init;
use sbi_rt::*;
use spin::{Lazy, Mutex};

use crate::arch::puts;

pub fn print(args: Arguments) {
    puts(args, false);
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::cons::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::cons::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}
