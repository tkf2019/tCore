mod logger;
mod panic;

use core::fmt::Arguments;
pub use logger::init;

use crate::arch::puts;

#[inline]
pub fn print(args: Arguments) {
    puts(args, false);
}

#[inline]
pub fn eprint(args: Arguments) {
    puts(args, true);
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::cons::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        $crate::cons::eprint(core::format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::cons::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => {
        $crate::cons::eprint(core::format_args!($($arg)*));
        $crate::eprintln!();
    }
}
