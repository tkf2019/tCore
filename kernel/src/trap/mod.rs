mod trampoline;
mod trapframe;

use core::arch::asm;
use log::info;
use log::trace;
use riscv::register::{scause::*, utvec::TrapMode, *};

pub use trampoline::__trampoline;
pub use trapframe::TrapFrame;

use crate::{
    config::TRAMPOLINE_VA,
    syscall::syscall,
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
    assert!(status.spp() == sstatus::SPP::User);
    // Handle user trap with detailed cause
    trace!(
        "USER TRAP {:X?}, {:X?}, {:#X}, {:#X}",
        cause, status, tval, epc
    );
    match cause {
        Trap::Exception(Exception::UserEnvCall) => {
            let current = current_task();
            let trapframe = current.unwrap().trapframe();

            trapframe.next_epc();

            match syscall(trapframe.syscall_args().unwrap()) {
                Ok(ret) => trapframe.set_a0(ret),
                Err(errno) => trapframe.set_a0(errno.try_into().unwrap()),
            };
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            let current = current_task();
            let trapframe = current.unwrap().trapframe();

            trace!("{:#X?}", trapframe);
            unimplemented!()
        }
        Trap::Exception(Exception::IllegalInstruction) => {}
        Trap::Interrupt(Interrupt::SupervisorTimer) => {}
        Trap::Interrupt(Interrupt::SupervisorExternal) => {}
        _ => {
            panic!("Unsupported trap {:?}!", cause);
        }
    }
    user_trap_return();
}

#[no_mangle]
pub fn user_trap_return() -> ! {
    extern "C" {
        fn uservec();
        fn userret();
    }
    unsafe {
        sstatus::clear_sie();
        stvec::write(TRAMPOLINE_VA as usize, TrapMode::Direct);
        let (satp, trapframe_base, userret_entry) = {
            let current = current_task().unwrap();
            let current_mm = current.mm.lock();
            (
                current_mm.page_table.satp(),
                trapframe_base(current.tid),
                userret as usize - uservec as usize + TRAMPOLINE_VA,
            )
        };
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
