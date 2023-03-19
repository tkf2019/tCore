#![allow(unused)]

use alloc::vec::Vec;
use bit_field::BitField;
use id_alloc::*;
use kernel_sync::SpinLock;
use spin::Lazy;
pub use syscall::*;
pub use uintr::*;

use crate::{
    arch::mm::{AllocatedFrame, PAGE_SIZE},
    error::KernelError,
};

const DEFAULT_UIST_SIZE: usize = 1;

const UISTE_VEC_MASK: u64 = 0xffff << 16;

const UISTE_INDEX_MASK: u64 = 0xffff << 48;

/// User interrupt sender status.
pub struct UIntrSender {
    /// Maximum number of frames.
    limit: usize,

    /// Sender status table allocator.
    alloc: RecycleAllocator,

    /// Frame allocated for send status table
    frames: Vec<AllocatedFrame>,
}

impl UIntrSender {
    /// Creates a new sender table.
    pub fn new(pages: usize) -> Self {
        let mut frames = Vec::new();
        frames.resize_with(pages, || AllocatedFrame::new(true).unwrap());
        Self {
            limit: PAGE_SIZE * pages / core::mem::size_of::<UISTE>(),
            alloc: RecycleAllocator::new(0),
            frames,
        }
    }

    /// Gets an entry by index.
    pub fn get(&self, index: usize) -> Option<&'static mut UISTE> {
        if index > self.limit {
            return None;
        }
        let pa = self.frames.first().unwrap().start_address().value()
            + index * core::mem::size_of::<UISTE>();
        Some(unsafe { &mut *(pa as *mut UISTE) })
    }

    /// Allocates a new [`UISTE`].
    pub fn alloc(&mut self) -> Option<usize> {
        let new = self.alloc.alloc();
        if new < self.limit { Some(new) } else { None }
    }

    /// Deallocates a [`UISTE`].
    pub fn dealloc(&mut self, index: usize) {
        if index < self.limit {
            self.alloc.dealloc(index);
        }
    }
}

/// User interrupt send status table entry.
pub struct UISTE(u64);

impl UISTE {
    /// Returns if this entry is valid.
    pub fn is_valid(&self) -> bool {
        (self.0 >> 63) != 0
    }

    /// Enables or disables this entry.
    pub fn set_valid(&mut self, valid: bool) {
        if valid {
            self.0 |= 1u64 << 63;
        } else {
            self.0 &= !(1u64 << 63);
        }
    }

    /// Sets sender vector of this entry.
    pub fn set_vec(&mut self, vec: usize) {
        self.0 &= !UISTE_VEC_MASK;
        self.0 |= ((vec as u64) << 16) & UISTE_VEC_MASK;
    }

    /// Gets sender vector of this entry.
    pub fn get_vec(&self) -> usize {
        ((self.0 & UISTE_VEC_MASK) >> 16) as usize
    }

    /// Sets receiver index of this entry.
    pub fn set_index(&mut self, index: usize) {
        self.0 &= !UISTE_INDEX_MASK;
        self.0 |= ((index as u64) << 48) & UISTE_INDEX_MASK;
    }

    /// Gets receiver index of this entry.
    pub fn get_index(&self) -> usize {
        ((self.0 & UISTE_INDEX_MASK) >> 48) as usize
    }
}

/// Global allocator
static UINTR_RECEIVER_ALLOC: Lazy<SpinLock<RecycleAllocator>> =
    Lazy::new(|| SpinLock::new(RecycleAllocator::new(0)));

/// User interrupt receiver tracker.
pub struct UIntrReceiverTracker(pub usize);

impl UIntrReceiverTracker {
    pub fn new() -> Self {
        let new = UINTR_RECEIVER_ALLOC.lock().alloc();
        assert!(new < 512);
        Self(new)
    }
}

impl Drop for UIntrReceiverTracker {
    fn drop(&mut self) {
        UINTR_RECEIVER_ALLOC.lock().dealloc(self.0);
    }
}

/// User interrupt receiver status in UINTC
#[repr(C)]
#[derive(Debug)]
pub struct UIntrReceiver {
    /// Kernel defined architecture mode and valid bit.
    mode: u16,

    /// The integer ID of the hardware thread running the code.
    hartid: u16,

    /// Reserved bits.
    _reserved: u32,

    /// One bit for each user interrupt vector. There is user-interrupt request for a vector if the corresponding bit is 1.
    irq: u64,
}

impl UIntrReceiver {
    /// Gets a [`UIntrReceiver`] from UINTC by index.
    pub fn from(index: usize) -> Self {
        assert!(index < UINTC_ENTRY_NUM);
        let low = uintc_read_low(index);
        let high = uintc_read_high(index);
        Self {
            mode: low as u16,
            hartid: (low >> 16) as u16,
            _reserved: 0,
            irq: high,
        }
    }

    /// Synchronize UINTC with this [`UIntrReceiver`].
    pub fn sync(&self, index: usize) {
        let low = (self.mode as u64) | ((self.hartid as u64) << 16);
        let high = self.irq;
        uintc_write_low(index, low);
        uintc_write_high(index, high);
    }
}

/// Task inner member for user interrupt status.
pub struct TaskUIntrInner {
    /// Sender status
    pub uist: Option<UIntrSender>,

    /// Receiver status
    pub uirs: Option<UIntrReceiverTracker>,

    /// Sender vector mask
    pub mask: u64,

    /// User interrupt entry
    pub utvec: usize,

    /// User interrupt handler
    pub uscratch: usize,
}

impl TaskUIntrInner {
    pub fn new() -> Self {
        Self {
            uist: None,
            uirs: None,
            mask: 0,
            utvec: 0,
            uscratch: 0,
        }
    }

    /// Allocates a sender vector.
    pub fn alloc(&mut self) -> usize {
        let i = self.mask.leading_ones() as usize;
        self.mask.set_bit(i, true);
        i
    }

    /// Deallocates a sender vector
    pub fn dealloc(&mut self, i: usize) {
        self.mask.set_bit(i, false);
    }
}

/// UINTC base
pub const UINTC_BASE: usize = 0x2F1_0000;

/// UINTC size
pub const UINTC_SIZE: usize = 0x4000;

/// Maximum number of UINTC entries
pub const UINTC_ENTRY_NUM: usize = 512;

/// UINTC register width
pub const UINTC_WIDTH: usize = 32;

/* UINTC operations */
pub const UINTC_SEND_OFF: usize = 0x00;
pub const UINTC_LOW_OFF: usize = 0x08;
pub const UINTC_HIGH_OFF: usize = 0x10;
pub const UINTC_ACT_OFF: usize = 0x18;

#[inline(never)]
pub fn uintc_send_uipi(index: usize) {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_SEND_OFF;
    unsafe { *(pa as *mut u64) = 1 };
}
#[inline(never)]
pub fn uintc_read_low(index: usize) -> u64 {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_LOW_OFF;
    unsafe { *(pa as *const u64) }
}
#[inline(never)]
pub fn uintc_write_low(index: usize, data: u64) {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_LOW_OFF;
    unsafe { *(pa as *mut u64) = data };
}
#[inline(never)]
pub fn uintc_read_high(index: usize) -> u64 {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_HIGH_OFF;
    unsafe { *(pa as *const u64) }
}
#[inline(never)]
pub fn uintc_write_high(index: usize, data: u64) {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_HIGH_OFF;
    unsafe { *(pa as *mut u64) = data };
}
#[inline(never)]
pub fn uintc_get_active(index: usize) -> bool {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_ACT_OFF;
    unsafe { *(pa as *const u64) == 0x1 }
}
#[inline(never)]
pub fn uintc_set_active(index: usize) {
    assert!(index < UINTC_ENTRY_NUM);
    let pa = UINTC_BASE + index * UINTC_WIDTH + UINTC_ACT_OFF;
    unsafe { *(pa as *mut u64) = 0x1 };
}

mod syscall {
    use alloc::sync::Arc;
    use bit_field::BitField;
    use errno::Errno;
    use riscv::register::sstatus;
    use syscall_interface::SyscallResult;
    use uintr::utvec::Utvec;
    use vfs::File;

    use crate::{
        arch::get_cpu_id,
        syscall::SyscallImpl,
        task::{cpu, Task},
    };

    use super::*;

    impl SyscallImpl {
        pub fn uintr_register_receier() -> SyscallResult {
            let curr = cpu().curr.as_ref().unwrap();

            if curr.uintr_inner().uirs.is_some() {
                return Err(Errno::EINVAL);
            }

            curr.uintr_inner().uirs = Some(UIntrReceiverTracker::new());

            // flush pending bits (low bits will be set during trap return).
            let uintr_inner = curr.uintr_inner();

            // init receiver status in UINTC
            let index = uintr_inner.uirs.as_ref().unwrap().0;
            let mut uirs = UIntrReceiver::from(index);
            uirs.irq = 0;
            uirs.sync(index);

            // save user status
            uintr_inner.utvec = utvec::read().bits();
            uintr_inner.uscratch = uscratch::read();

            Ok(0)
        }

        pub fn uintr_create_fd(vector: usize) -> SyscallResult {
            let curr = cpu().curr.as_ref().unwrap();
            if let Some(uirs) = &curr.uintr_inner().uirs {
                if !curr.uintr_inner().mask.get_bit(vector) {
                    curr.uintr_inner().mask.set_bit(vector, true);
                    let fd = curr.files().push(Arc::new(UIntrFile {
                        uirs_index: uirs.0,
                        vector,
                    }))?;
                    return Ok(fd);
                } else {
                    return Err(Errno::EINVAL);
                }
            }
            Err(Errno::EINVAL)
        }

        pub fn uintr_register_sender(fd: usize) -> SyscallResult {
            let curr = cpu().curr.as_ref().unwrap();
            let file = curr.files().get(fd)?;
            if file.is_uintr() {
                if curr.uintr_inner().uist.is_none() {
                    curr.uintr_inner().uist = Some(UIntrSender::new(1));
                }

                let uist = curr.uintr_inner().uist.as_mut().unwrap();
                if let Some(index) = uist.alloc() {
                    let uiste = uist.get(index).unwrap();
                    let file = file.as_any().downcast_ref::<UIntrFile>().unwrap();
                    uiste.set_valid(true);
                    uiste.set_vec(file.vector);
                    uiste.set_index(file.uirs_index);
                    return Ok(index);
                } else {
                    return Err(Errno::EINVAL);
                }
            }
            Err(Errno::EINVAL)
        }
    }

    /// Synchronize receiver status to UINTC and raise user interrupt if kernel returns to
    /// a receiver with pending interrupt requests.
    pub unsafe fn uirs_sync() {
        let uintr_inner = cpu().curr.as_ref().unwrap().uintr_inner();
        if let Some(uirs) = &uintr_inner.uirs {
            let index = uirs.0;
            let mut uirs = UIntrReceiver::from(index);
            uirs.hartid = get_cpu_id() as u16;
            uirs.mode |= 0x2; // 64 bits
            uirs.sync(index);

            log::trace!("uirs_sync {:x} {:x?}", index, uirs);

            // user configurations
            utvec::write(uintr_inner.utvec, utvec::TrapMode::Direct);
            uscratch::write(uintr_inner.uscratch);
            uie::set_usoft();

            // supervisor configurations
            suirs::write((1 << 63) | (index & 0xffff));
            sideleg::set_usoft();
            if uirs.irq != 0 {
                sip::set_usoft();
            } else {
                sip::clear_usoft();
            }
        } else {
            // supervisor configurations
            suirs::write(0);
            sideleg::clear_usoft();
            sip::clear_usoft();
        }
    }

    /// Initialize starting frame number of sender status table.
    pub fn uist_init() {
        let curr = cpu().curr.as_ref().unwrap();
        if let Some(uist) = &curr.uintr_inner().uist {
            log::trace!("uist_init {:x?}", uist.frames);

            uintr::suist::write(
                (1 << 63) | (1 << 44) | uist.frames.first().unwrap().start_address().value(),
            );
        }
    }

    /// Called during trap return.
    pub fn uintr_return() {
        // receiver
        unsafe { uirs_sync() };

        // sender
        uist_init();
    }

    pub struct UIntrFile {
        pub uirs_index: usize,
        pub vector: usize,
    }

    impl File for UIntrFile {
        fn is_uintr(&self) -> bool {
            true
        }
    }
}

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
    log::info!("Send UIPI!");
    uipi_send(0);

    loop {
        if uintr::sip::read().usoft() {
            log::info!("Receive UINT!");
            uintr::sip::clear_usoft();
            assert!(uipi_read() == (0x00010000 | (1 << hartid)));
            break;
        }
    }
}

pub const UINTR_TESTCASES: &[&str] = &[
    "argv",
    "uipi_sample",
    "pthread_cancel_points",
    "pthread_cancel",
];
