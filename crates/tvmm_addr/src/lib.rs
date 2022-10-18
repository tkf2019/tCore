#![feature(step_trait)]
#![allow(unused_imports)]

#[macro_use]
extern crate derive_more;
extern crate paste;

use core::{
    cmp::{max, min},
    fmt,
    iter::Step,
    ops::*,
};
use paste::paste;

/// A macro for defining `VirtualAddress` and `PhysicalAddress` structs and implementing their
/// common traits.
#[macro_export]
macro_rules! implement_address {
    (
        $TypeName:ident,
        $desc:literal,
        $prefix:literal,
        $is_canonical:ident,
        $canonicalize:ident,
        $chunk:ident,
        $page_size:expr
    ) => {
        paste! {

            #[doc = "A " $desc " memory address, which is a `usize` under the hood."]
            #[derive(
                Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
                Binary, Octal, LowerHex, UpperHex,
                BitAnd, BitOr, BitXor, BitAndAssign, BitOrAssign, BitXorAssign,
                Add, Sub, AddAssign, SubAssign,
            )]
            #[repr(transparent)]
            pub struct $TypeName(usize);

            impl $TypeName {
                #[doc = "Creates a new `" $TypeName "`, returning an error if the address is
                    not canonical. This is useful for checking whether an address is valid
                    before using it."]
                pub fn new(addr: usize) -> Option<$TypeName> {
                    if $is_canonical(addr) { Some($TypeName(addr)) } else { None }
                }

                #[doc = "Creates a new `" $TypeName "` that is guaranteed to be canonical."]
                pub const fn new_canonical(addr: usize) -> $TypeName {
                    $TypeName($canonicalize(addr))
                }

                #[doc = "Creates a new `" $TypeName "` with a value 0."]
                pub const fn zero() -> $TypeName {
                    $TypeName(0)
                }

                #[doc = "Returns the underlying `usize` value for this `" $TypeName "`."]
                #[inline]
                pub const fn value(&self) -> usize {
                    self.0
                }

                #[doc = "Returns the offset from the " $chunk " boundary specified by this `"
                    $TypeName ".\n\n \
                    For example, if the [`PAGE_SIZE`] is 4096 (4KiB), then this will return
                    the least significant 12 bits `(12:0]` of this `" $TypeName "`."]
                pub const fn [<$chunk _offset>](&self) -> usize {
                    self.0 & ($page_size - 1)
                }

                #[doc = "Returns if the address is aligned or not."]
                pub const fn is_aligned(&self) -> bool {
                    self.[<$chunk _offset>]() == 0
                }
            }
            impl fmt::Debug for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, concat!($prefix, "{:#X}"), self.0)
                }
            }
            impl fmt::Display for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl fmt::Pointer for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl Add<usize> for $TypeName {
                type Output = $TypeName;
                fn add(self, rhs: usize) -> $TypeName {
                    $TypeName::new_canonical(self.0.saturating_add(rhs))
                }
            }
            impl AddAssign<usize> for $TypeName {
                fn add_assign(&mut self, rhs: usize) {
                    *self = $TypeName::new_canonical(self.0.saturating_add(rhs));
                }
            }
            impl Sub<usize> for $TypeName {
                type Output = $TypeName;
                fn sub(self, rhs: usize) -> $TypeName {
                    $TypeName::new_canonical(self.0.saturating_sub(rhs))
                }
            }
            impl SubAssign<usize> for $TypeName {
                fn sub_assign(&mut self, rhs: usize) {
                    *self = $TypeName::new_canonical(self.0.saturating_sub(rhs));
                }
            }
            impl From<usize> for $TypeName {
                fn from(addr: usize) -> Self {
                    $TypeName($canonicalize(addr))
                }
            }
            impl Into<usize> for $TypeName {
                #[inline]
                fn into(self) -> usize {
                    self.0
                }
            }
        }
    };
}

/// A macro for defining `Page` and `Frame` structs
/// and implementing their common traits, which are generally identical.
#[macro_export]
macro_rules! implement_page_frame {
    (
        $TypeName:ident,
        $desc:literal,
        $prefix:literal,
        $address:ident,
        $page_size:expr,
        $max_page_number:expr
    ) => {
        paste! {

            #[doc = "A `" $TypeName "` is a chunk of **" $desc "** memory aligned to a[`PAGE_SIZE`] boundary."]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            pub struct $TypeName {
                number: usize,
            }

            impl $TypeName {
                #[doc = "Returns the `" $address "` at the start of this `" $TypeName "`."]
                pub const fn start_address(&self) -> $address {
                    $address::new_canonical(self.number * $page_size)
                }

                #[doc = "Returns the number of this `" $TypeName "`."]
                #[inline(always)]
                pub const fn number(&self) -> usize {
                    self.number
                }

                #[doc = "Returns the `" $TypeName "` containing the given `" $address "`."]
                pub const fn containing_address(addr: $address) -> $TypeName {
                    $TypeName {
                        number: addr.value() / $page_size,
                    }
                }

                #[doc = "Returns the `" $TypeName "` containing the given `" $address "`."]
                pub const fn ceil(addr: $address) -> $TypeName {
                    $TypeName {
                        number: addr.value() / $page_size,
                    }
                }

                #[doc = "Returns the next `" $TypeName "` not containing the given `" $address
                    "`."]
                pub const fn floor(addr: $address) -> $TypeName {
                    $TypeName {
                        number: (addr.value() -1 + $page_size) / $page_size,
                    }
                }
            }
            impl From<usize> for $TypeName {
                fn from(number: usize) -> Self {
                    Self {
                        number,
                    }
                }
            }
            impl Into<usize> for $TypeName {
                #[inline(always)]
                fn into(self) -> usize {
                    self.number
                }
            }
            impl From<$address> for $TypeName {
                fn from(addr: $address) -> Self {
                    $TypeName::ceil(addr)
                }
            }
            impl Into<$address> for $TypeName {
                fn into(self) -> $address{
                    self.start_address()
                }
            }
            impl fmt::Debug for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(
                        f,
                        concat!(stringify!($TypeName), "(", $prefix, "{:#X})"),
                        self.start_address()
                    )
                }
            }
            impl Add<usize> for $TypeName {
                type Output = $TypeName;
                fn add(self, rhs: usize) -> $TypeName {
                    // cannot exceed max page number (which is also max frame number)
                    $TypeName {
                        number: core::cmp::min(
                            $max_page_number, self.number.saturating_add(rhs)),
                    }
                }
            }
            impl AddAssign<usize> for $TypeName {
                fn add_assign(&mut self, rhs: usize) {
                    *self = $TypeName {
                        number: core::cmp::min(
                            $max_page_number, self.number.saturating_add(rhs)),
                    };
                }
            }
            impl Sub<usize> for $TypeName {
                type Output = $TypeName;
                fn sub(self, rhs: usize) -> $TypeName {
                    $TypeName {
                        number: self.number.saturating_sub(rhs),
                    }
                }
            }
            impl SubAssign<usize> for $TypeName {
                fn sub_assign(&mut self, rhs: usize) {
                    *self = $TypeName {
                        number: self.number.saturating_sub(rhs),
                    };
                }
            }
            #[doc = "Implementing `Step` allows `" $TypeName "` to be used in an [`Iterator`]."]
            impl Step for $TypeName {
                #[inline]
                fn steps_between(start: &$TypeName, end: &$TypeName) -> Option<usize> {
                    Step::steps_between(&start.number, &end.number)
                }
                #[inline]
                fn forward_checked(start: $TypeName, count: usize) -> Option<$TypeName> {
                    Step::forward_checked(start.number, count).map(|n| $TypeName { number: n })
                }
                #[inline]
                fn backward_checked(start: $TypeName, count: usize) -> Option<$TypeName> {
                    Step::backward_checked(start.number, count).map(|n| $TypeName { number: n })
                }
            }
        }
    };
}

/// A macro for defining `PageRange` and `FrameRange` structs
/// and implementing their common traits, which are generally identical.
#[macro_export]
macro_rules! implement_page_frame_range {
    (
        $TypeName:ident,
        $desc:literal,
        $short:ident,
        $chunk:ident,
        $address:ident,
        $page_size:expr
    ) => {
        paste! {

            #[doc = "A range of [`" $chunk "`]s that are contiguous in " $desc " memory."]
            #[derive(Clone, PartialEq, Eq)]
            pub struct $TypeName(RangeInclusive<$chunk>);

            impl $TypeName {
                #[doc = "Creates a new range of [`" $chunk "`]s that spans from `start` to 
                    `end`, both inclusive bounds."]
                pub const fn new(start: $chunk, end: $chunk) -> $TypeName {
                    $TypeName(RangeInclusive::new(start, end))
                }

                #[doc = "Creates a `" $TypeName "` that will always yield `None` when iterated.\
                    "]
                pub const fn empty() -> $TypeName {
                    $TypeName::new($chunk { number: 1 }, $chunk { number: 0 })
                }

                #[doc = "A convenience method for creating a new `" $TypeName "` that spans \
                    all [`" $chunk "`]s from the given [`" $address "`] to an end bound based \
                    on the given size."]
                pub fn [<from_ $short _addr>](
                    start_addr: $address,
                    size_in_bytes: usize
                ) -> $TypeName {
                    assert!(size_in_bytes > 0);
                    let start = $chunk::containing_address(start_addr);
                    // The end bound is inclusive, hence the -1. Parentheses are needed to
                    // avoid overflow.
                    let end = $chunk::containing_address(start_addr + (size_in_bytes - 1));
                    $TypeName::new(start, end)
                }

                #[doc = "Returns the [`" $address "`] of the starting [`" $chunk "`] in this \
                    `" $TypeName "`."]
                pub const fn start_address(&self) -> $address {
                    self.0.start().start_address()
                }

                #[doc = "Returns the number of [`" $chunk "`]s covered by this iterator.\n\n \
                    Use this instead of [`Iterator::count()`] method. \
                    This is instant, because it doesn't need to iterate over each entry, \
                    unlike normal iterators."]
                pub const fn [<size_in_ $chunk:lower s>](&self) -> usize {
                    // add 1 because it's an inclusive range
                    (self.0.end().number + 1).saturating_sub(self.0.start().number)
                }

                /// Returns the size of this range in number of bytes.
                pub const fn size_in_bytes(&self) -> usize {
                    self.[<size_in_ $chunk:lower s>]() * $page_size
                }

                #[doc = "Returns `true` if this `" $TypeName "` contains the given \
                    [`" $address "`]."]
                pub fn contains_address(&self, addr: $address) -> bool {
                    self.0.contains(&$chunk::containing_address(addr))
                }

                #[doc = "Returns the offset of the given [`" $address "`] within this \
                    `" $TypeName "`, \
                    i.e., `addr - self.start_address()`.\n\n \
                    If the given `addr` is not covered by this range of [`" $chunk "`]s, \
                    this returns `None`.\n\n \
                    # Examples\n \
                    If the range covers addresses `0x2000` to `0x4000`, then `offset_of_address
                    (0x3500)` would return `Some(0x1500)`."]
                pub fn offset_of_address(&self, addr: $address) -> Option<usize> {
                    if self.contains_address(addr) {
                        Some(addr.value() - self.start_address().value())
                    } else {
                        None
                    }
                }

                #[doc = "Returns the [`" $address "`] at the given `offset` into this \
                    `" $TypeName "`within this `" $TypeName "`, \
                    i.e., `addr - self.start_address()`.\n\n \
                    If the given `offset` is not within this range of [`" $chunk "`]s, \
                    this returns `None`.\n\n \
                    # Examples\n \
                    If the range covers addresses `0x2000` to `0x4000`, then `address_at_offset
                    (0x1500)` would return `Some(0x3500)`."]
                pub fn address_at_offset(&self, offset: usize) -> Option<$address> {
                    if offset <= self.size_in_bytes() {
                        Some(self.start_address() + offset)
                    }
                    else {
                        None
                    }
                }

                #[doc = "Returns a new separate `" $TypeName "` that is extended to include \
                    the given [`" $chunk "`]."]
                pub fn to_extended(&self, to_include: $chunk) -> $TypeName {
                    // if the current range was empty, return a new range containing only the
                    // given page/frame
                    if self.is_empty() {
                        return $TypeName::new(to_include.clone(), to_include);
                    }
                    let start = min(self.0.start(), &to_include);
                    let end = max(self.0.end(), &to_include);
                    $TypeName::new(start.clone(), end.clone())
                }

                #[doc = "Returns an inclusive `" $TypeName "` representing the [`" $chunk "`]s \
                    that overlap across this `" $TypeName "` and the given other \
                    `" $TypeName "`.\n\n \
                    If there is no overlap between the two ranges, `None` is returned."]
                pub fn overlap(&self, other: &$TypeName) -> Option<$TypeName> {
                    let starts = max(*self.start(), *other.start());
                    let ends   = min(*self.end(),   *other.end());
                    if starts <= ends {
                        Some($TypeName::new(starts, ends))
                    } else {
                        None
                    }
                }
            }
            impl fmt::Debug for $TypeName {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{:?}", self.0)
                }
            }
            impl Deref for $TypeName {
                type Target = RangeInclusive<$chunk>;
                fn deref(&self) -> &RangeInclusive<$chunk> {
                    &self.0
                }
            }
            impl DerefMut for $TypeName {
                fn deref_mut(&mut self) -> &mut RangeInclusive<$chunk> {
                    &mut self.0
                }
            }
            impl IntoIterator for $TypeName {
                type Item = $chunk;
                type IntoIter = RangeInclusive<$chunk>;
                fn into_iter(self) -> Self::IntoIter {
                    self.0
                }
            }
        }
    };
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    fn is_canonical_va(va: usize) -> bool {
        true
    }
    const fn canonicalize_va(va: usize) -> usize {
        va
    }

    implement_address!(
        VirtAddr,
        "virtual",
        "v",
        is_canonical_va,
        canonicalize_va,
        page,
        0x1000
    );

    implement_page_frame!(Page, "virtual", "v", VirtAddr, 0x1000, usize::MAX / 0x1000);

    implement_page_frame_range!(PageRange, "physical", virt, Page, VirtAddr, 0x1000);

    #[test]
    fn it_works() {
        let mut va = VirtAddr::new(0).unwrap();
        va += 0x1001;
        assert!(va.page_offset() == 1);
        assert!(va != VirtAddr::new(0x1000).unwrap());

        let p = Page::from(va);
        let v: VirtAddr = p.into();
        assert!(p.number() == 1);
        assert!(Page::ceil(v) == Page::from(1));

        let pr = PageRange::from_virt_addr(0x1002.into(), 0x3124);
        let mut iter = pr.into_iter().map(|p| p.number());
        assert!(iter.next() == Some(1));
        assert!(iter.next() == Some(2));
        assert!(iter.next() == Some(3));
        assert!(iter.next() == Some(4));
        assert!(iter.next() == None);
    }
}