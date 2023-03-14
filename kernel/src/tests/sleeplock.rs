use kernel_sync::SleepLock;
use log::debug;
use spin::Lazy;

use crate::task::{cpu, TaskLockedInner};

pub static LOCKED_DATA: Lazy<SleepLock<usize, TaskLockedInner>> = Lazy::new(|| SleepLock::new(0));

pub fn test() {
    let mut locked_data = LOCKED_DATA.lock(&cpu().curr.as_ref().unwrap().locked_inner);
    *locked_data += 1;
    debug!("LOCKED_DATA {}", *locked_data);
    drop(locked_data);
}
