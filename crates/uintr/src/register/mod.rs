pub use riscv::register::{ucause, uepc, uie, uip, uscratch, ustatus, utval, utvec};

pub mod suitt;
pub mod upidaddr;

#[macro_export]
macro_rules! read_csr {
    ($csr_number:literal) => {
        /// Reads the CSR
        #[inline]
        unsafe fn _read() -> usize {
            let r: usize;
            core::arch::asm!(concat!("csrrs {0}, ", stringify!($csr_number), ", x0"), out(reg) r);
            r
        }
    };
}

#[macro_export]
macro_rules! write_csr {
    ($csr_number:literal) => {
        /// Writes the CSR
        #[inline]
        #[allow(unused_variables)]
        unsafe fn _write(bits: usize) {
            core::arch::asm!(concat!("csrrw x0, ", stringify!($csr_number), ", {0}"), in(reg) bits);
        }
    };
}
