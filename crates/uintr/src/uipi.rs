use alloc::format;

use crate::uuirq;

pub unsafe fn uipi_send(index: usize) {
    core::arch::asm!(".insn i 0x7b, 0x0, x0, {}, 0x0", in(reg) index);
}

pub unsafe fn uipi_read() -> usize {
    core::arch::asm!(".insn i 0x7b, 0x1, x0, x0, 0x0");
    uuirq::read()
}

pub unsafe fn uipi_write(bits: usize) {
    uuirq::write(bits);
    core::arch::asm!(".insn i 0x7b, 0x2, x0, x0, 0x0");
}

pub unsafe fn uipi_activate() {
    core::arch::asm!(".insn i 0x7b, 0x3, x0, x0, 0x0");
}

pub unsafe fn uipi_deactivate() {
    core::arch::asm!(".insn i 0x7b, 0x4, x0, x0, 0x0");
}