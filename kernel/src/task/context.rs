/// Saved registers for kernel context switch
#[repr(C)]
#[derive(Debug)]
pub struct TaskContext {
    //. return address
    ra: usize,

    /// stack pointer
    sp: usize,

    /// callee-saved register
    s: [usize; 12],
}

impl TaskContext {
    /// Create a new [`TaskContext`] with the entry of the trap exit and kernel stack
    /// allocated for this task.
    pub fn new(trap_return: usize, kstack_base: usize) -> Self {
        Self {
            ra: trap_return,
            sp: kstack_base,
            s: [0; 12],
        }
    }

    /// A zero task context
    pub fn zero() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}

/// Switch task context
#[naked]
#[no_mangle]
pub unsafe extern "C" fn switch(curr: *const TaskContext, next: *const TaskContext) {
    core::arch::asm!(
        // Save return address of current flow
        "sd ra, 0(a0)",
        // Save kernel stack of current task
        "sd sp, 8(a0)",
        // Save callee-saved registers
        "
        sd s0, 16(a0)
        sd s1, 24(a0)
        sd s2, 32(a0)
        sd s3, 40(a0)
        sd s4, 48(a0)
        sd s5, 56(a0)
        sd s6, 64(a0)
        sd s7, 72(a0)
        sd s8, 80(a0)
        sd s9, 88(a0)
        sd s10, 96(a0)
        sd s11, 104(a0)
        ",
        // Restore return address of next flow
        "ld ra, 0(a1)",
        // Restore kernel stack of next flow
        "ld sp, 8(a1)",
        // Restore callee-saved registers
        "
        ld s0, 16(a1)
        ld s1, 24(a1)
        ld s2, 32(a1)
        ld s3, 40(a1)
        ld s4, 48(a1)
        ld s5, 56(a1)
        ld s6, 64(a1)
        ld s7, 72(a1)
        ld s8, 80(a1)
        ld s9, 88(a1)
        ld s10, 96(a1)
        ld s11, 104(a1)
        ",
        // Return as if nothing happened
        "ret",
        options(noreturn)
    );
}
