mod logger;
mod panic;

use core::fmt::{Arguments, Result, Write};
use sbi_rt::*;

pub use logger::init;

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            #[allow(deprecated)]
            legacy::console_putchar(c as _);
        }
        Ok(())
    }
}

pub fn print(args: Arguments) {
    Console.write_fmt(args).unwrap();
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
