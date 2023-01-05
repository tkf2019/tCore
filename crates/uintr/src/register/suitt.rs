//! suitt register

use bit_field::BitField;

pub struct Suitt {
    bits: usize,
}

impl Suitt {
    /// Returns the contents of the register as raw bits
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }

    /// User-interrupt enabled.
    #[inline]
    pub fn enabled(&self) -> bool {
        self.bits.get_bit(31)
    }

    /// Physical page number.
    #[inline]
    pub fn ppn(&self) -> usize {
        self.bits.get_bits(0..44)
    }

    /// UITT size.
    pub fn size(&self) -> usize {
        self.bits.get_bits(44..56)
    }
}

read_csr_as!(Suitt, 0x1C0);
write_csr_as_usize!(0x1C0);
