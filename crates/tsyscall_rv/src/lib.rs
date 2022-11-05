#![no_std]

use core::arch::asm;
use tsyscall_no::SyscallNO;

#[inline(always)]
pub unsafe fn syscall0(id: SyscallNO) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        out("a0") ret,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall1(id: SyscallNO, a0: usize) -> isize {
    let ret: isize;
    asm!("ecall",
        inlateout("a0") a0 => ret,
        in("a7") id as usize,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall2(id: SyscallNO, a0: usize, a1: usize) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        inlateout("a0") a0 => ret,
        in("a1") a1,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall3(id: SyscallNO, a0: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        inlateout("a0") a0 => ret,
        in("a1") a1,
        in("a2") a2,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall4(id: SyscallNO, a0: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        inlateout("a0") a0 => ret,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall5(
    id: SyscallNO,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        inlateout("a0") a0 => ret,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
    );
    ret
}

#[inline(always)]
pub unsafe fn syscall6(
    id: SyscallNO,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> isize {
    let ret: isize;
    asm!("ecall",
        in("a7") id as usize,
        inlateout("a0") a0 => ret,
        in("a1") a1,
        in("a2") a2,
        in("a3") a3,
        in("a4") a4,
        in("a5") a5,
    );
    ret
}
