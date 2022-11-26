use core::fmt::{Arguments, Write};

use spin::{Lazy, Mutex};

use crate::println;

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

#[inline]
pub fn putchar(c: u8) {
    STDOUT.lock().putchar(c);
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
