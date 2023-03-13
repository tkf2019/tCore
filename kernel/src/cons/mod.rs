mod logger;
mod panic;

use core::fmt::{Arguments, Write};
use kernel_sync::SpinLock;
pub use logger::init;
use spin::Lazy;

struct Stdin;

impl Stdin {
    #[inline]
    #[allow(deprecated)]
    pub fn getchar(&self) -> u8 {
        sbi_rt::legacy::console_getchar() as _
    }
}

struct Stdout;

#[inline]
#[allow(deprecated)]
pub fn putchar(c: u8) {
    sbi_rt::legacy::console_putchar(c as _);
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            if c == 127 {
                putchar(8);
                putchar(b' ');
                putchar(8);
            } else {
                putchar(c);
            }
        }
        Ok(())
    }
}

static STDIN: Lazy<SpinLock<Stdin>> = Lazy::new(|| SpinLock::new(Stdin));
static STDOUT: Lazy<SpinLock<Stdout>> = Lazy::new(|| SpinLock::new(Stdout));
static STDERR: Lazy<SpinLock<Stdout>> = Lazy::new(|| SpinLock::new(Stdout));

#[inline]
pub fn getchar() -> u8 {
    STDIN.lock().getchar()
}

#[inline]
pub fn stdout_puts(args: Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

#[inline]
pub fn stderr_puts(args: Arguments) {
    let _stdout = STDOUT.try_lock();
    STDERR.lock().write_fmt(args).unwrap();
}

#[inline]
pub fn print(args: Arguments) {
    stdout_puts(args);
}

#[inline]
pub fn eprint(args: Arguments) {
    stderr_puts(args);
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
