mod trampoline;
mod trapframe;

use core::{arch::asm, panic};
use log::trace;
use riscv::register::{scause::*, utvec::TrapMode, *};

pub use trampoline::__trampoline;
pub use trapframe::TrapFrame;

use crate::task::do_exit;
use crate::{
    config::TRAMPOLINE_VA,
    syscall::syscall,
    task::{manager::current_task, trapframe_base},
};

use self::trapframe::KernelTrapContext;

/// Set kernel trap entry.
pub fn set_kernel_trap() {
    extern "C" {
        fn __kernelvec();
    }
    unsafe {
        stvec::write(
            __kernelvec as usize - __trampoline as usize + TRAMPOLINE_VA,
            TrapMode::Direct,
        );
        sscratch::write(kernel_trap_handler as usize);
    }
}

/// Set user trap entry.
pub fn set_user_trap() {
    unsafe { stvec::write(TRAMPOLINE_VA as usize, TrapMode::Direct) };
}

#[no_mangle]
pub fn user_trap_handler() -> ! {
    set_kernel_trap();

    let scause = scause::read();
    let sstatus = sstatus::read();
    let stval = stval::read();
    let sepc = sepc::read();
    // Only handle user trap
    assert!(sstatus.spp() == sstatus::SPP::User);

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // pc + 4
            let current = current_task().unwrap();
            let trapframe = current.trapframe();
            trapframe.next_epc();
            // Syscall may change the flow
            drop(current);
            match syscall(trapframe.syscall_args().unwrap()) {
                Ok(ret) => trapframe.set_a0(ret),
                Err(errno) => trapframe.set_a0(errno.try_into().unwrap()),
            };
        }
        _ => {
            // Handle user trap with detailed cause
            trace!(
                "User trap {:X?}, {:X?}, {:#X}, {:#X}",
                scause.cause(),
                sstatus,
                stval,
                sepc
            );
            do_exit(-1);
        }
    }
    user_trap_return();
}

#[no_mangle]
pub fn user_trap_return() -> ! {
    extern "C" {
        fn __uservec();
        fn __userret();
    }
    let (satp, trapframe_base, userret) = {
        let current = current_task().unwrap();
        let current_mm = current.mm.lock();
        (
            current_mm.page_table.satp(),
            trapframe_base(current.tid),
            __userret as usize - __uservec as usize + TRAMPOLINE_VA,
        )
    };

    set_user_trap();

    unsafe {
        asm!(
            "fence.i",
            "jr {userret}",
            userret = in(reg) userret,
            in("a0") trapframe_base,
            in("a1") satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn kernel_trap_handler(ctx: &KernelTrapContext) -> ! {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        _ => {
            panic!(
                "Kernel trap {:X?}, stval = {:#X}, ctx = {:#X?} ",
                scause.cause(),
                stval,
                ctx
            );
        }
    }
}
