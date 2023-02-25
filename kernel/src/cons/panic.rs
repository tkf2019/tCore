use core::panic::PanicInfo;
use kernel_sync::SpinLock;
use sbi_rt::*;
use spin::Lazy;

use crate::{
    arch::get_cpu_id,
    config::CPU_NUM,
    println,
};

static PANIC_COUNT: Lazy<SpinLock<usize>> = Lazy::new(|| SpinLock::new(0));

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "\u{1B}[31m[CPU{:>3}] Panicked at {}:{} {}\u{1B}[0m",
            get_cpu_id(),
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!(
            "\u{1B}[31m[CPU{:>3}] Panicked at {}\u{1B}[0m",
            get_cpu_id(),
            info.message().unwrap()
        );
    }

    let mut panic_count = PANIC_COUNT.lock();
    *panic_count += 1;
    if *panic_count == CPU_NUM {
        println!("All CPU panicked! Shuttting down...");
        system_reset(Shutdown, SystemFailure);
    }
    drop(panic_count);
    loop {}
}
