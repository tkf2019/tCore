mod trampoline;
mod trapframe;

use core::arch::asm;
use log::debug;
use riscv::register::{scause::*, utvec::TrapMode, *};

pub use trampoline::trampoline;

use crate::{
    config::TRAMPOLINE_VA,
    task::{manager::current_task, trapframe_base},
};

#[no_mangle]
pub fn user_trap_handler() -> ! {
    // set kernel trap entry
    let cause = scause::read().cause();
    let status = sstatus::read();
    let tval = stval::read();
    let epc = sepc::read();
    // Only handle user trap
    assert!(status.spp() != sstatus::SPP::User);
    // Handle user trap with detailed cause
    match cause {
        Trap::Exception(Exception::UserEnvCall) => {}
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            debug!("
                [kernel] Killed user task due to {:?}: bad address = {:#x}, bad instruction = {:#x}",
                cause,
                tval,
                epc,
            );
        }
        Trap::Exception(Exception::IllegalInstruction) => {}
        Trap::Interrupt(Interrupt::SupervisorTimer) => {}
        Trap::Interrupt(Interrupt::SupervisorExternal) => {}
        _ => {
            panic!("Unsupported trap {:?}!", cause);
        }
    }
    unreachable!()
}

#[no_mangle]
pub fn user_trap_return() -> ! {
    extern "C" {
        fn uservec();
        fn userret();
    }
    unsafe {
        sstatus::clear_sie();
        stvec::write(TRAMPOLINE_VA, TrapMode::Direct);
        let current = current_task().lock();
        let satp = current.mm.page_table.satp();
        let trapframe_base = trapframe_base(current.tid);
        let userret_entry = userret as usize - uservec as usize + TRAMPOLINE_VA;
        drop(current);
        asm!(
            "fence.i",
            "jr {userret_entry}",
            userret_entry = in(reg) userret_entry,
            in("a0") trapframe_base,
            in("a1") satp,
            options(noreturn)
        );
    }
}
