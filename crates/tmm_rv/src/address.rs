use tmm_addr::*;

use super::config::*;

/// Load and store effective addresses, which are 64 bits, must have bits 63–39 all equal to
/// bit 38, or else an address exception will occur.
#[inline]
fn is_canonical_va(va: usize) -> bool {
    let high_bits: usize = va & !VA_MASK_SV39;
    let bit_38: usize = va & VA_38_SV39;
    high_bits == !VA_MASK_SV39 && bit_38 != 0 || high_bits == 0 && bit_38 == 0
}

#[inline]
fn is_canonical_pa(pa: usize) -> bool {
    (pa & !PA_MASK_SV39) == 0
}

#[inline]
const fn canonicalize_va(va: usize) -> usize {
    ((va << (64 - VA_BITS_SV39)) as isize >> (64 - VA_BITS_SV39)) as usize
}

#[inline]
const fn canonicalize_pa(pa: usize) -> usize {
    pa & PA_MASK_SV39
}

implement_address!(
    VirtAddr,
    "virtual",
    "v",
    is_canonical_va,
    canonicalize_va,
    "page",
    PAGE_SIZE
);
implement_address!(
    PhysAddr,
    "physical",
    "p",
    is_canonical_pa,
    canonicalize_pa,
    "frame",
    PAGE_SIZE
);

implement_page_frame!(Page, "virtual", VirtAddr, PAGE_SIZE, MAX_VA / PAGE_SIZE);
implement_page_frame!(Frame, "physical", PhysAddr, PAGE_SIZE, MAX_VA / PAGE_SIZE);

implement_page_frame_range!(PageRange, "virtual", virt, Page, VirtAddr, PAGE_SIZE);
implement_page_frame_range!(FrameRange, "physical", phys, Frame, PhysAddr, PAGE_SIZE);

impl Page {
    pub fn split_vpn(&self) -> [usize; 3] {
        let mut vpn = self.number();
        let mut indexes = [0usize; 3];
        for i in (0..3).rev() {
            indexes[i] = vpn & ((1 << INDEX_BITS_SV39) - 1);
            vpn >>= INDEX_BITS_SV39;
        }
        indexes
    }
}

impl PhysAddr {
    /// Returns a immutable reference of `T` starting from the physical address.
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *const T).as_ref().unwrap() }
    }

    /// Returns a mutable reference of `T` starting from the physical address.
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
