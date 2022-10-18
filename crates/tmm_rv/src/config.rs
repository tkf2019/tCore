/// Support 4 KB page
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xC;

/// Max physical address width in SV39
pub const PA_BTIS_SV39: usize = 56;
pub const PPN_BITS_SV39: usize = PA_BTIS_SV39 - PAGE_SIZE_BITS;

/// Max virtual address width in SV39
/// Bits [63:39] must be set the same as bit 38.
/// Virtual space can only use the highest and lowest 256 GB.
pub const VA_BITS_SV39: usize = 39;
pub const VPN_BITS_SV39: usize = VA_BITS_SV39 - PAGE_SIZE_BITS;

/// One beyond the highest possible virtual address allowed by SV39.
pub const MAX_VA: usize = 1 << (9 + 9 + 9 + 12 - 1);

/// 3-level page table in SV39
pub const PT_LEVELS_SV39: usize = 3;
pub const INDEX_BITS_SV39: usize = 9;

/// Bit masks in SV39
pub const VA_MASK_SV39: usize = 0x0000_007F_FFFF_FFFF;
pub const VA_38_SV39: usize = 0x0000_0040_0000_0000;
pub const PA_MASK_SV39: usize = 0x0003_FFFF_FFFF_FFFF;
