cfg_if::cfg_if! {
    if #[cfg(all(target_os = "none", any(target_arch = "riscv64")))] {
        #[allow(dead_code)]
        use riscv::register::sstatus;

        /// Gets CPU id from `tp` register. Remember to avoid using
        /// `tp` in your kernel.
        pub(crate) fn cpu_id() -> usize {
            let mut cpu_id;
            unsafe {
                core::arch::asm!("mv {0}, tp", out(reg) cpu_id);
            }
            cpu_id
        }

        /// Interrupt on
        pub(crate) fn intr_on() {
            unsafe { sstatus::set_sie() };
        }

        /// Interrupt off
        pub(crate) fn intr_off() {
            unsafe { sstatus::clear_sie() };
        }

        /// Gets if interrupt is enabled
        pub(crate) fn intr_get() -> bool {
            sstatus::read().sie()
        }
    } else {
        pub(crate) fn cpu_id() -> usize {
            0
        }

        pub(crate) fn intr_on() {}

        pub(crate) fn intr_off() {}

        pub(crate) fn intr_get() -> bool {
            false
        }
    }
}
