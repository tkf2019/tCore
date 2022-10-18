use bitflags::*;

bitflags! {
    /// Page table entry flag bits
    pub struct PTEFlags: u8 {
        /// Iff V = 1, the entry is valid.
        const V = 1 << 0;
        /// If R = 1, the virtual page is readable.
        const R = 1 << 1;
        /// If W = 1, the virtual page is writable.
        const W = 1 << 2;
        /// If X = 1, the virtual page is executable.
        const X = 1 << 3;
        /// If U = 1, the virtual page is accessible in user privilege.
        const U = 1 << 4;
        /// If G = 1, the virtual page is accessible in all privileges.
        const G = 1 << 5;
        /// If the entry is recently accessed.
        const A = 1 << 6;
        /// If the entry is recently modified.
        const D = 1 << 7;
    }
}

/// Page Table Entry:
/// - 63:54 -> Reserved
/// - 53:28 -> PPN\[2\]
/// - 27:19 -> PPN\[1\]
/// - 18:10 -> PPN\[0\]
/// - 9:8   -> RSW
/// - 7:0   -> Flags
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PTE {
    pub bits: usize,
}

// impl PTE {
//     /// Generate PTE from physical page number and flags
//     pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
//         PTE {
//             bits: ppn.0 << 10 | flags.bits as usize,
//         }
//     }
//     /// Get physical page number
//     pub fn ppn(&self) -> PhysPageNum {
//         (self.bits >> 10 & ((1usize << 44) - 1)).into()
//     }
//     /// Get page table entry flags
//     pub fn flags(&self) -> PTEFlags {
//         PTEFlags::from_bits(self.bits as u8).unwrap()
//     }
// }

