use core::{
    cell::SyncUnsafeCell,
    fmt,
    marker::PhantomData,
    mem::{align_of, size_of, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic,
};

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{SeqLock, SpinLock};

/// Changes a [`RcuType`] to a thin pointer.
///
/// # Safety
///
/// This function might cause memory leak without help of compiler.
#[must_use]
#[inline(always)]
fn rcu_into<T: RcuType>(value: T) -> usize {
    T::check();

    use core::mem::transmute_copy;
    const USIZE_SIZE: usize = size_of::<usize>();
    let v = unsafe {
        match size_of::<T>() {
            1 => transmute_copy::<T, u8>(&value) as usize,
            2 => transmute_copy::<T, u16>(&value) as usize,
            4 => transmute_copy::<T, u32>(&value) as usize,
            USIZE_SIZE => transmute_copy::<T, usize>(&value),
            size => unreachable!("Unsupported size: {}", size),
        }
    };
    core::mem::forget(value);
    v
}

/// Changes the thin pointer back to the [`RcuType`].
#[inline(always)]
fn rcu_from<T: RcuType>(value: usize) -> T {
    T::check();

    use core::mem::transmute_copy;
    const USIZE_SIZE: usize = size_of::<usize>();
    unsafe {
        match size_of::<T>() {
            1 => transmute_copy(&(value as u8)),
            2 => transmute_copy(&(value as u16)),
            4 => transmute_copy(&(value as u32)),
            USIZE_SIZE => transmute_copy(&value),
            size => unreachable!("Unsupported size: {}", size),
        }
    }
}

/// A wrapper collecting the thin pointer and its drop handler for memory reclamation.
pub struct RcuDrop(usize, unsafe fn(usize));

impl Drop for RcuDrop {
    fn drop(&mut self) {
        panic!("Leaked RcuDrop");
    }
}

impl RcuDrop {
    /// This function calls the handler directly. Thus the space taken by this [`RcuDrop`]
    /// is meaningless and we just ignore it.
    #[inline(always)]
    pub unsafe fn release(self) {
        self.1(self.0);
        core::mem::forget(self);
    }
}

pub type RcuDropFn = fn(RcuDrop);

/// Global drop function for [`RcuDrop`], which should be provided before using [`RcuType`] types.
static mut RCU_DROP_FN: Option<RcuDropFn> = None;

/// Global pending [`RcuDrop`]s.
static RCU_DROP_PENDING: SpinLock<Vec<RcuDrop>> = SpinLock::new(Vec::new());

/// Global reclamation to handle pending [`RcuDrop`]s with drop function provided,
pub unsafe fn reclamation(rcu_drop_fn: RcuDropFn) {
    RCU_DROP_FN.replace(rcu_drop_fn);
    let v = core::mem::take(&mut *RCU_DROP_PENDING.lock());
    v.into_iter().for_each(rcu_drop_fn);
}

/// Clears the handler and waits for next [`reclamation`].
pub unsafe fn wait() {
    RCU_DROP_FN.take();
}

/// Drops [`RcuType`] types manually, calling the [`RCU_DROP_FN`] immediately if a
#[inline]
fn rcu_drop<T: RcuType>(x: T) {
    if !core::mem::needs_drop::<T>() {
        return;
    }

    match unsafe { RCU_DROP_FN } {
        Some(drop_fn) => unsafe { drop_fn(x.transmute()) },
        None => RCU_DROP_PENDING.lock().push(unsafe { x.transmute() }),
    }
}

/// Types implementing [`RcuType`] must be able to load or store just by one instruction.
/// Thus the size of a [`RcuType`] cannot exceed the size of [`usize`]. Alignment should be
/// the same as the size, e.g. `Box<dyn T>` is a type that cannot implement RCU. For these
/// types, a [`SeqLock`] can be applied instead. If there is no need to deconstruct the type,
/// it will not be pushed to the pending list.
///
/// The basic idea behind RCU is to split updates into **removal** and **reclamation** phases:
/// - **removal**: The removal phase removes references to data items within a data structure
/// (possibly by replacing them with references to new versions of these data items), and can
/// run concurrently with readers. The reason that it is safe to run the removal phase concurrently
/// with readers is the semantics of modern CPUs guarantee that readers will see either the old or
/// the new version of the data structure rather than a partially updated reference.
/// In this implementation, [`core::ptr::replace`] satisfies the need perfectly.
/// - **reclamation**: The reclamation phase does the work of reclaiming (e.g., freeing) the data
/// items removed from the data structure during the removal phase. Because reclaiming data items
/// can disrupt any readers concurrently referencing those data items, the reclamation phase must
/// not start until readers no longer hold references to those data items.
pub trait RcuType: Sized + 'static {
    /// Checks the type before using RCU.
    #[inline(always)]
    fn check() {
        assert_eq!(size_of::<Self>(), align_of::<Self>());
        assert!(size_of::<Self>() <= size_of::<usize>());
        assert!(size_of::<u32>() <= size_of::<usize>());
    }

    /// Wait for all previous readers to complete their RCU read-side critical sections. At this point,
    /// there cannot be any readers who hold references to the data structure, so it now may safely be reclaimed
    #[inline]
    fn reclamation(self) {
        self::rcu_drop(self)
    }

    /// Transmute the ownership to [`RcuDrop`].
    #[must_use]
    #[inline(always)]
    unsafe fn transmute(self) -> RcuDrop {
        RcuDrop(rcu_into(self), |a| unsafe { drop(rcu_from::<Self>(a)) })
    }

    /// Read that precedes any write which follows it in program order returns a guard.
    #[must_use]
    #[inline(always)]
    fn read(&self) -> RcuReadGuard<Self> {
        Self::check();

        // Inhibits compiler from automatically calling T's destructor.
        // Volatile operations are intended to act on I/O memory, and are guaranteed to not
        // be elided or reordered by the compiler across other volatile operations.
        let data = unsafe { core::mem::ManuallyDrop::new(core::ptr::read_volatile(self)) };

        // An acquire fence prevents the memory reordering of any read which precedes it in program order
        // with any read or write which follows it in program order, usually used after a read.
        atomic::fence(atomic::Ordering::Acquire);

        RcuReadGuard {
            data,
            _mark: PhantomData,
        }
    }

    /// # Safety
    ///
    /// This function provides no synchronization guarantees and should be protected by a lock.
    #[inline]
    unsafe fn write(&self, new: Self) {
        Self::check();

        // A release fence prevents the memory reordering of any read or write which precedes it
        // in program order with any write which follows it in program order, usually used before a write.
        atomic::fence(atomic::Ordering::Release);

        // removal phase
        let old = core::ptr::replace(self as *const _ as *mut Self, new);

        // reclamation phase
        old.reclamation();
    }

    /// Atomic write without extra synchronization considerations.
    #[inline]
    fn write_atomic(&self, src: Self) {
        Self::check();

        unsafe {
            // new data will not be dropped by compiler
            let new = rcu_into(src);

            // A atomic implementation for core::ptr::replace, returning the old value.
            macro_rules! atomic_swap_impl {
                ($at: ident, $ut: ty) => {{
                    use core::sync::atomic::$at;
                    core::mem::transmute::<_, &$at>(self)
                        .swap(new as $ut, atomic::Ordering::Release) as usize
                }};
            }

            // removal phase
            let old = match size_of::<Self>() {
                1 => atomic_swap_impl!(AtomicU8, u8),
                2 => atomic_swap_impl!(AtomicU16, u16),
                4 => atomic_swap_impl!(AtomicU32, u32),
                8 => atomic_swap_impl!(AtomicU64, u64),
                _ => panic!(),
            };

            // reclamation phase
            rcu_from::<Self>(old).reclamation();
        }
    }
}

impl<T: 'static> RcuType for *const T {}
impl<T: 'static> RcuType for *mut T {}
impl<T: 'static> RcuType for NonNull<T> {}
impl<T: 'static> RcuType for Box<T> {}
impl<T: 'static> RcuType for Arc<T> {}
impl<T: 'static> RcuType for Weak<T> {
    #[inline(always)]
    fn reclamation(self) {
        // no need to drop when Weak points to nothing
        if !self.ptr_eq(&Weak::new()) {
            self::rcu_drop(self)
        }
    }
}

/// Implements [`RcuType`] methods for types wrapped by [`Option`].
macro_rules! option_rcu_impl {
    ($T: ident, $name: ty) => {
        impl<$T: 'static> RcuType for $name {
            #[inline(always)]
            fn reclamation(self) {
                if let Some(p) = self {
                    self::rcu_drop(p);
                }
            }
        }
    };
}

option_rcu_impl!(T, Option<NonNull<T>>);
option_rcu_impl!(T, Option<Box<T>>);
option_rcu_impl!(T, Option<Arc<T>>);
option_rcu_impl!(T, Option<Weak<T>>);

/// A guard that provides immutable access to inner data.
pub struct RcuReadGuard<'a, T: RcuType> {
    _mark: PhantomData<&'a T>,
    data: ManuallyDrop<T>,
}

impl<'a, T: RcuType> !Send for RcuReadGuard<'a, T> {}
impl<'a, T: RcuType> !Sync for RcuReadGuard<'a, T> {}

impl<'a, T: RcuType> Deref for RcuReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

/// Inner [`RcuType`] wrapped with [`SyncUnsafeCell`].
pub struct RcuCell<T: RcuType>(SyncUnsafeCell<T>);

impl<T: RcuType> RcuCell<T> {
    /// Creates a new [`RcuCell`].
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self(SyncUnsafeCell::new(value))
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`RcuCell`] mutably, and a mutable reference is guaranteed to be exclusive in Rust,
    /// no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist. As such,
    /// this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }

    /// Returns a guard that permits immutable access to inner data.
    ///
    /// # Safety
    ///
    /// Avoid holding multiple guards at the same time, for the data may be different.
    #[inline(always)]
    pub fn read(&self) -> RcuReadGuard<T> {
        unsafe { &*self.0.get() }.read()
    }

    /// # Safety
    ///
    /// This function provides no synchronization guarantees and should be protected by a lock.
    #[inline]
    pub unsafe fn write(&self, src: T) {
        (&*self.0.get()).write(src)
    }

    /// Atomic write without extra synchronization considerations.
    #[inline]
    pub fn write_atomic(&self, src: T) {
        unsafe { (&*self.0.get()).write_atomic(src) }
    }
}
