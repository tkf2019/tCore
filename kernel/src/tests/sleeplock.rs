use kernel_sync::SleepLock;
use log::debug;
use spin::Lazy;

use crate::task::{curr_task, TaskLockedInner};

pub static LOCKED_DATA: Lazy<SleepLock<usize, TaskLockedInner>> = Lazy::new(|| SleepLock::new(0));

pub fn test() {
    let mut locked_data = LOCKED_DATA.lock(&curr_task().unwrap().locked_inner);
    *locked_data += 1;
    debug!("LOCKED_DATA {}", *locked_data);
    drop(locked_data);
}
