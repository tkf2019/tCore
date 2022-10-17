mod trampoline;
mod trapframe;

use log::debug;
use riscv::register::{scause::*, *};

#[no_mangle]
fn user_trap_handler() -> ! {
    // set kernel trap entry
    let cause = scause::read().cause();
    let status = sstatus::read();
    let tval = stval::read();
    let epc = sepc::read();
    // Only handle user trap
    assert!(status.spp() != sstatus::SPP::User);
    // Handle user trap with detailed cause
    match cause {
        Trap::Exception(Exception::UserEnvCall) => {
            
        }
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
