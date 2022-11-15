#![allow(unused)]

/// We only supports 4 KB page.
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 12;

/// Max physical address width in SV39
pub const PA_BTIS_SV39: usize = 56;

/// Designated as `PA [55:12]`
pub const PPN_BITS_SV39: usize = PA_BTIS_SV39 - PAGE_SIZE_BITS;

/// Physical space can only use the lowest 2^56 bytes.
pub const PA_MASK_SV39: usize = 0x0003_FFFF_FFFF_FFFF;

/// Max virtual address width in SV39.
pub const VA_BITS_SV39: usize = 39;

/// Designated as `VA [38:12]`.
pub const VPN_BITS_SV39: usize = VA_BITS_SV39 - PAGE_SIZE_BITS;

/// Virtual space can only use the highest and lowest 256 GB.
pub const VA_MASK_SV39: usize = 0x0000_007F_FFFF_FFFF;

/// Bits [63:39] must be set the same as bit 38.
pub const VA_38_SV39: usize = 0x0000_0040_0000_0000;

/// The highest possible virtual address.
pub const MAX_VA: usize = usize::MAX;

/// The highest virtual address of the low 256 GB in SV39.
// pub const LOW_MAX_VA: usize = 0x0000_003F_FFFF_FFFF;
pub const LOW_MAX_VA: usize = 0xFFFF_FFFF;

/// 3-level page table in SV39.
pub const PAGE_TABLE_LEVELS_SV39: usize = 3;

/// 9-bit vpn for 3-level page table in SV39.
pub const INDEX_BITS_SV39: usize = 9;

/// PPN bits in page table entry in SV39.
pub const PPN_MASK_SV39: usize = 0x003F_FFFF_FFFF_FC00;

/// PPN offset in page table entry in SV39.
pub const PPN_OFFSET_SV39: usize = 10;

/// Flag bits in page table entry.
pub const FLAG_MASK_SV39: usize = 0x0000_0000_0000_00FF;

/// `satp` mode
pub const SATP_MODE_SV39: usize = 0x8000_0000_0000_0000;
