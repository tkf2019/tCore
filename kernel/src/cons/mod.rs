mod logger;
mod panic;

use core::fmt::{Arguments, Write};
pub use logger::init;
use spin::{Lazy, Mutex};

struct Stdin;

impl Stdin {
    #[inline]
    #[allow(deprecated)]
    pub fn getchar(&self) -> u8 {
        sbi_rt::legacy::console_getchar() as _
    }
}

struct Stdout;

impl Stdout {
    #[inline]
    #[allow(deprecated)]
    pub fn putchar(&self, c: u8) {
        sbi_rt::legacy::console_putchar(c as _);
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.putchar(c);
        }
        Ok(())
    }
}

static STDIN: Lazy<Mutex<Stdin>> = Lazy::new(|| Mutex::new(Stdin));
static STDOUT: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(Stdout));
static STDERR: Lazy<Mutex<Stdout>> = Lazy::new(|| Mutex::new(Stdout));

#[inline]
pub fn getchar() -> u8 {
    STDIN.lock().getchar()
}

/// Stderr has higher priority than Stdout.
#[inline]
pub fn puts(args: Arguments, err: bool) {
    if err {
        let _ = STDOUT.try_lock();
        STDERR.lock().write_fmt(args).unwrap();
    } else {
        STDOUT.lock().write_fmt(args).unwrap();
    }
}

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
