use core::{panic::PanicInfo, intrinsics::unreachable};
use log::error;
use sbi_rt::*;
use spin::{Lazy, Mutex};

use crate::{get_cpu_id, config::CPU_NUM};

static PANIC_COUNT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("Panicked at {}", info.message().unwrap());
    }

    let mut panic_count = PANIC_COUNT.lock();
    *panic_count += 1;
    if *panic_count == CPU_NUM {
        error!("All CPU panicked! Shuttting down...");
        system_reset(Shutdown, SystemFailure);
    }
    drop(panic_count);
    
    loop {}
    unreachable!()
}
