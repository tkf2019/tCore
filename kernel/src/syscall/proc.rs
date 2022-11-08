use crate::{
    error::{KernelError, KernelResult},
    task::manager::current_task,
};

pub fn getpid() -> KernelResult<Errno> {
    let current = current_task();
    let current = current.lock();
    Ok(current.pid)
}
