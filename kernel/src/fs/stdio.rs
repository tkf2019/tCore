//! File descriptors are a part of the POSIX API. Each Unix process (except perhaps
//! daemons) should have three standard POSIX file descriptors, corresponding to the
//! three standard streams:
//!
//! - 0: Standard input (STDIN)
//! - 1: Standard output (STDOUT)
//! - 2: Standard error (STDERR)

use vfs::File;

use crate::{cons::getchar, eprint, print, task::do_yield};

pub struct Stdin;

impl File for Stdin {
    fn read(&self, buf: &mut [u8]) -> Option<usize> {
        if buf.len() == 0 {
            return Some(0);
        }
        buf[0] = loop {
            let c = getchar();
            if c == 0 || c == 255 {
                unsafe { do_yield() };
                continue;
            } else {
                break c;
            }
        };
        Some(1)
    }

    fn read_ready(&self) -> bool {
        true
    }
}

pub struct Stdout;

impl File for Stdout {
    fn write(&self, buf: &[u8]) -> Option<usize> {
        if let Ok(data) = core::str::from_utf8(buf) {
            print!("{}", data);
            Some(buf.len())
        } else {
            None
        }
    }

    fn write_ready(&self) -> bool {
        true
    }
}

pub struct Stderr;

impl File for Stderr {
    fn write(&self, buf: &[u8]) -> Option<usize> {
        if let Ok(data) = core::str::from_utf8(buf) {
            eprint!("{}", data);
            Some(buf.len())
        } else {
            None
        }
    }

    fn write_ready(&self) -> bool {
        true
    }
}
