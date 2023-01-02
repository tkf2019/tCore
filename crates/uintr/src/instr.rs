#[naked]
pub unsafe extern "C" fn uret() {
    core::arch::asm!("uret", options(noreturn));
}

// #[naked]
// pub unsafe extern "C" fn senduipi() {
//     core::arch::asm!("
//     .quad
//         0x12345678
//     ", options(noreturn))
// }