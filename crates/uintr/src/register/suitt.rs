//! suitt register

use crate::{read_csr, write_csr};

write_csr!(0x1C0);
read_csr!(0x1C0);

#[inline]
pub fn read() -> usize {
    unsafe { _read() }
}

#[inline]
pub fn write(bits: usize) {
    unsafe { _write(bits) };
}
