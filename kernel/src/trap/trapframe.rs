use riscv::register::sstatus::Sstatus;
use tmm_rv::PhysAddr;
use tsyscall::SyscallNO;

use crate::{
    error::{KernelError, KernelResult},
    syscall::SyscallArgs,
};

/// User context is saved in trapframe by trap handler in trampoline.
#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    /// Kernel page table root
    kernel_satp: usize,
    /// Kernel stack pointer
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
    /// Saved hartid
    cpu_id: usize,
}

impl TrapFrame {
    /// Create a new trap frame with user stack pointer.
    pub fn new(
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
        user_epc: usize,
        user_status: Sstatus,
        user_sp: usize,
        cpu_id: usize,
    ) -> Self {
        let mut trapframe = Self {
            kernel_satp,
            kernel_sp,
            trap_handler,
            user_epc,
            user_status,
            user_regs: [0; 31],
            cpu_id,
        };
        trapframe.user_regs[1] = user_sp;
        trapframe
    }

    /// Get syscall arguments in registers in user trap frame.
    ///
    /// Returns error if syscall number not supported.
    pub fn syscall_args(&self) -> KernelResult<SyscallArgs> {
        Ok(SyscallArgs(
            SyscallNO::try_from(self.user_regs[16])
                .map_err(|no| KernelError::SyscallUnsupported(no))?,
            [
                self.user_regs[9],  // x10
                self.user_regs[10], // x11
                self.user_regs[11], // x12
                self.user_regs[12], // x13
                self.user_regs[13], // x14
                self.user_regs[14], // x15
            ],
        ))
    }

    /// Step to next instruction after the trap instruction.
    pub fn next_epc(&mut self) {
        self.user_epc += 4;
    }

    /// Returns mutable reference of a trapframe
    pub fn from(pa: PhysAddr) -> &'static mut TrapFrame {
        unsafe { (pa.value() as *mut TrapFrame).as_mut().unwrap() }
    }

    /// Set return errno or value after an syscall.
    pub fn set_a0(&mut self, a0: usize) {
        self.user_regs[9] = a0;
    }
}


/// Kernel trap context is saved on the kernel stack.
#[repr(C)]
#[derive(Debug)]
pub struct KernelTrapContext {
    regs: [usize; 29],
    sepc: usize,
    sstatus: Sstatus,
}
