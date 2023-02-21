use log::info;
pub use uintr::*;

use crate::{arch::mm::AllocatedFrame, println};

/// UINTC base
pub const UINTC_BASE: usize = 0x2F1_0000;

/// UINTC size
pub const UINTC_SIZE: usize = 0x4000;

/// Test user interrupt implementation.
/// 1. Test CSRs: suicfg, suirs, suist
/// 2. Test UINTC: Write to UINTC directly
/// 3. Test UIPI: READ, WRITE, SEND
#[allow(unused)]
pub unsafe fn test_uintr(hartid: usize) {
    suicfg::write(UINTC_BASE);
    assert_eq!(suicfg::read(), UINTC_BASE);

    // Enable receiver status.
    let uirs_index = hartid;
    // Receiver on hart hartid
    *((UINTC_BASE + uirs_index * 0x20 + 8) as *mut u64) = ((hartid << 16) as u64) | 3;
    suirs::write((1 << 63) | uirs_index);
    assert_eq!(suirs::read().bits(), (1 << 63) | uirs_index);
    // Write to high bits
    uipi_write(0x00010000);
    assert!(uipi_read() == 0x00010000);

    // Enable sender status.
    let frame = AllocatedFrame::new(true).unwrap();
    suist::write((1 << 63) | (1 << 44) | frame.number());
    assert_eq!(suist::read().bits(), (1 << 63) | (1 << 44) | frame.number());
    // valid entry, uirs index = hartid, sender vector = hartid
    *(frame.start_address().value() as *mut u64) = ((hartid << 48) | (hartid << 16) | 1) as u64;
    // Send uipi with first uist entry
    info!("Send UIPI!");
    uipi_send(0);

    loop {
        if uintr::sip::read().usoft() {
            info!("Receive UINT!");
            uintr::sip::clear_usoft();
            assert!(uipi_read() == (0x00010000 | (1 << hartid)));
            break;
        }
    }
}
