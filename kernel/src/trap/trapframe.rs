use riscv::register::sstatus::Sstatus;

/// User context is saved in trapframe for the trap handling code in trampoline.
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    /// Kernel page table root
    kernel_satp: usize,
    /// Kernel stack poit0nter
    kernel_sp: usize,
    /// Trap handler address
    trap_handler: usize,
    /// User program counter
    user_epc: usize,
    /// User status
    user_status: Sstatus,
    /// Saved global registers (arch dependent)
    /// No need to save x0 (wired to zero)
    user_regs: [usize; 31],
}

impl TrapFrame {
    pub fn new(
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
        user_epc: usize,
        user_status: Sstatus,
        user_sp: usize,
    ) -> Self {
        let mut trapframe = Self {
            kernel_satp,
            kernel_sp,
            trap_handler,
            user_epc,
            user_status,
            user_regs: [0; 31],
        };
        trapframe.user_regs[1] = user_sp;
        trapframe
    }
}
