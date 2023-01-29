#![no_std]
#![no_main]
#![feature(naked_functions, asm_const)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]

mod config;
mod cons;
mod driver;
mod error;
mod fs;
mod heap;
mod loader;
mod mm;
mod syscall;
mod task;
mod timer;
mod trap;

extern crate alloc;

use log::{info, trace};
use riscv::asm::{sfence_vma, sfence_vma_all};
use tmm_rv::{frame_init, Frame, PhysAddr, VirtAddr};
use uintr::{uipi_send, uipi_activate};

use crate::config::{
    BOOT_STACK_SIZE, CPU_NUM, FLASH_BASE, IS_TEST_ENV, PHYSICAL_MEMORY_END, TOTAL_BOOT_STACK_SIZE,
    UINTC_BASE, VIRTIO0,
};

// Initialize kernel stack in .bss section.
#[link_section = ".bss.stack"]
static mut STACK: [u8; TOTAL_BOOT_STACK_SIZE] = [0u8; TOTAL_BOOT_STACK_SIZE];

/// Entry for the first kernel.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn __entry(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "la sp, {stack} + {stack_size}",
        // Jump to the main function.
        "j  {main}",
        stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack      =   sym STACK,
        main       =   sym rust_main,
        options(noreturn),
    )
}

/// Entry for other kernels.
#[naked]
#[no_mangle]
pub unsafe extern "C" fn __entry_others(hartid: usize) -> ! {
    core::arch::asm!(
        // Use tp to save hartid
        "mv tp, a0",
        // Set stack pointer to the kernel stack.
        "
        la a1, {stack}
        li t0, {total_stack_size}
        li t1, {stack_size}
        mul sp, a0, t1
        sub sp, t0, sp
        add sp, a1, sp
        ",
        // Jump to the main function.
        "j  {main}",
        total_stack_size = const TOTAL_BOOT_STACK_SIZE,
        stack_size       = const BOOT_STACK_SIZE,
        stack            =   sym STACK,
        main             =   sym rust_main_others,
        options(noreturn),
    )
}

/// Get cpu id.
#[inline]
pub fn get_cpu_id() -> usize {
    let cpu_id;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) cpu_id) };
    cpu_id
}

/// Start other harts
#[inline]
pub fn start_hart(hartid: usize, entry: usize, opaque: usize) {
    let ret = sbi_rt::hart_start(hartid, entry, opaque);
    assert!(ret.is_ok(), "Failed to shart hart {}", hartid);
}

/// Flush tlb
pub fn flush_tlb(va: Option<VirtAddr>) {
    if let Some(va) = va {
        unsafe { sfence_vma(0, va.value()) };
    } else {
        unsafe { sfence_vma_all() };
    }
}

/// Clear .bss
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize) -> ! {
    clear_bss();
    cons::init();
    // Initialize global heap allocator.
    heap::init();
    // Initialize global frame allocator.
    extern "C" {
        fn ekernel();
    }
    frame_init(
        Frame::ceil(PhysAddr::from(ekernel as usize)).into(),
        Frame::floor(PhysAddr::from(PHYSICAL_MEMORY_END)).into(),
    );
    // Set kernel trap entry.
    trap::set_kernel_trap();
    // Activate kernel virtual address space.
    mm::init();
    // Initialize oscomp testcases, which will be loaded from disk.
    if IS_TEST_ENV {
        oscomp::init(oscomp::testcases::LIBC_STATIC_TESTCASES);
    }
    // unsafe { uintr::test::test_register() };
    // Initialize the first task.
    task::init();

    // Wake up other harts.
    for cpu_id in 0..CPU_NUM {
        if cpu_id != hartid {
            let entry = __entry_others as usize;
            info!("Try to start hart {}", cpu_id);
            start_hart(cpu_id, entry, 0);
        }
    }

    // let uipi_addr = UINTC_BASE;
    // for i in 0..3 {
    //     unsafe {
    //         info!("Send uipi!");
    //         *((uipi_addr + i * 0x20 + 8) as *mut u64) = 0x00010003;
    //         *((uipi_addr + i * 0x20) as *mut u64) = 0x1;

    //         loop {
    //             if uintr::sip::read().usoft() {
    //                 info!("Receive UINT!");
    //                 uintr::sip::clear_usoft();
    //                 break;
    //             }
    //         }
    //     }
    // }

    // IDLE loop
    task::idle();
}

#[no_mangle]
pub extern "C" fn rust_main_others(_hartid: usize) -> ! {
    // Set kernel trap entry.
    trap::set_kernel_trap();
    // Activate kernel virtual address space.
    mm::init();
    info!("(Secondary) Start executing tasks.");

    // loop {
    //     if uintr::sip::read().usoft() {
    //         info!("Receive UINT!");
    //         unsafe { uintr::sip::clear_usoft() };

    //         info!("Send uipi!");
    //         unsafe {
    //             *((UINTC_BASE + 3 * 0x20 + 8) as *mut u64) = 0x00000003;
    //             *((UINTC_BASE + 3 * 0x20) as *mut u64) = 0x1;
    //         }
    //     }
    // }

    // IDLE loop
    task::idle();
}
