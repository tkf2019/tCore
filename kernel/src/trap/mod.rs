mod trampoline;
mod trapframe;

use riscv::register::*;

#[no_mangle]
fn user_trap_handler() -> ! {
    // set kernel trap entry
    // let cause = scause::read();
    let status = sstatus::read();
    // Only handle user trap
    assert!(status.spp() != sstatus::SPP::User);
    // TODO
    panic!("User trap!");
}
