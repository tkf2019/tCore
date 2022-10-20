use core::panic::PanicInfo;
use log::error;
use sbi_rt::*;

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
    system_reset(Shutdown, SystemFailure);
    unreachable!()
}
