//! A naive sleeping mutex.
//!
//! A preemptive kernel differs from a nonpreemptive kernel on the way a process running in Kernel Mode
//! reacts to asynchronous events that could induce a process switch—for instance, an interrupt handler
//! that awakes a higher priority process. We will call this kind of process switch a forced process switch.
//!
//! Thus we use sleep lock to avoid infinite latency.

use core::{
    cell::UnsafeCell,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

use id_alloc::{IDAllocator, RecycleAllocator};
use spin::Lazy;

use crate::{
    arch::{cpu_id, intr_get},
    spin::{SpinMutex, SpinMutexGuard},
    CPUs,
};

pub trait Sched {
    /// Locks current thread and returns the guard that permits inner access to the thread.
    ///
    /// This function requires manual unlocking before exiting the scope.
    fn lock(thread: &SpinMutex<Self>) -> *mut Self {
        unsafe {
            core::mem::forget(thread.lock());
            thread.as_mut_ptr()
        }
    }

    /// Unlocks current thread.
    fn unlock(thread: &SpinMutex<Self>) {
        unsafe { thread.force_unlock() };
    }

    /// Thread state is changed to `SLEEPING`.
    fn sleep(thread: *mut Self);

    /// Wakeup all threads sleeping on this lock.
    fn wakeup(id: usize);

    /// Threads acquiring this [`SleepMutex`] is grouped by the id.
    fn set_id(thread: *mut Self, id: Option<usize>);

    /// Switch to scheduler.
    unsafe fn sched();
}

static SleepMutexIDAllocator: Lazy<SpinMutex<RecycleAllocator>> =
    Lazy::new(|| SpinMutex::new(RecycleAllocator::new(0)));

/// A sleep lock providing mutually exclusive access to data and yielding the CPU when locked.
pub struct SleepMutex<T: ?Sized, S: Sched> {
    phantom: PhantomData<S>,

    /// [`SpinMutex`] protecting this [`SleepMutex`].
    inner: SpinMutex<SleepMutexInner<T, S>>,
}

/// Inner info protected by lock.
pub struct SleepMutexInner<T: ?Sized, S: Sched> {
    phantom: PhantomData<S>,

    /// A unique identifier of this [`SleepMutex`].
    id: usize,

    /// If this mutex is locked and holds the inner data.
    locked: bool,

    /// Data of this mutex.
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct SleepMutexGuard<'a, T: ?Sized + 'a, S: Sched> {
    phantom: PhantomData<S>,
    lock: &'a SpinMutex<SleepMutexInner<T, S>>,
    data: &'a mut T,
}

// unsafe thread-safe impls
unsafe impl<T: ?Sized + Send, S: Sched> Sync for SleepMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: Sched> Send for SleepMutex<T, S> {}

impl<T, S: Sched> SleepMutex<T, S> {
    /// Creates a new [`SleepMutex`] wrapping the supplied data.
    #[inline(always)]
    pub fn new(data: T) -> Self {
        SleepMutex {
            phantom: PhantomData,
            inner: SpinMutex::new(SleepMutexInner::new(data)),
        }
    }

    /// Consumes this [`SleepMutex`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let SleepMutex { inner, .. } = self;
        inner.into_inner().into_inner()
    }

    /// Returns a mutable pointer to the underlying data.
    ///
    /// This is mostly meant to be used for applications which require manual unlocking, but where
    /// storing both the lock and the pointer to the inner data gets inefficient.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        unsafe { (*self.inner.as_mut_ptr()).as_mut_ptr() }
    }
}

impl<T, S: Sched> SleepMutexInner<T, S> {
    /// Creates a new [`SleepMutexInner`].
    #[inline(always)]
    pub fn new(data: T) -> Self {
        SleepMutexInner {
            phantom: PhantomData,
            id: SleepMutexIDAllocator.lock().alloc(),
            locked: false,
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes this [`SleepMutexInner`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let SleepMutexInner { data, .. } = self;
        data.into_inner()
    }

    /// Returns a mutable pointer to the underlying data.
    ///
    /// This is mostly meant to be used for applications which require manual unlocking, but where
    /// storing both the lock and the pointer to the inner data gets inefficient.
    #[inline(always)]
    pub fn as_mut_ptr(&self) -> *mut T {
        self.data.get()
    }
}

impl<T: ?Sized, S: Sched> SleepMutex<T, S> {
    /// Locks the [`SleepMutex`], returning a guard that permits access to the inner data.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    #[inline(always)]
    pub fn lock(&self, thread: &SpinMutex<S>) -> SleepMutexGuard<T, S> {
        let mut inner = self.inner.lock();
        let lock_id = inner.id;

        // Automatically release the lock and sleep on chan.
        while inner.locked {
            // Must acquire thread lock in order to change thread state and then call sched().
            let thread_ptr = S::lock(thread);
            drop(inner);

            // Go to sleep
            S::sleep(thread_ptr);
            S::set_id(thread_ptr, Some(lock_id));

            unsafe {
                // Interrupt cannot be nesting or set before scheduler.
                assert!(CPUs[cpu_id()].noff == 1 && !intr_get());
                // Saves and restores CPU local variable, intena.
                let intena = CPUs[cpu_id()].intena;
                S::sched();
                CPUs[cpu_id()].intena = intena;
            }

            // Tidy up
            S::set_id(thread_ptr, None);

            // Reacquire original lock
            S::unlock(thread);
            inner = self.inner.lock();
        }
        inner.locked = true;

        SleepMutexGuard {
            phantom: PhantomData,
            lock: &self.inner,
            data: unsafe { &mut *inner.data.get() },
        }
    }

    /// Returns `true` if the lock is currently held.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.inner.lock().locked
    }

    /// Force unlock this [`SleepMutex`].
    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        let mut inner = self.inner.lock();
        inner.locked = false;
        S::wakeup(inner.id);
    }

    /// Tries to lock this [`SleepMutex`], returning a guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<SleepMutexGuard<T, S>> {
        let mut inner = self.inner.lock();
        if !inner.locked {
            inner.locked = true;
            Some(SleepMutexGuard {
                phantom: PhantomData,
                lock: &self.inner,
                data: unsafe { &mut *inner.data.get() },
            })
        } else {
            None
        }
    }
}

impl<T: ?Sized + fmt::Debug, S: Sched> fmt::Debug for SleepMutex<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SleepMutex {{ data: ")
                .and_then(|()| (&*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SleepMutex {{ <locked> }}"),
        }
    }
}

impl<T: ?Sized + Default, S: Sched> Default for SleepMutex<T, S> {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<T, S: Sched> From<T> for SleepMutex<T, S> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T: ?Sized, S: Sched> SleepMutexInner<T, S> {
    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`SleepMutexInner`] mutably, and a mutable reference is guaranteed to be exclusive in Rust,
    /// no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist. As such,
    /// this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        unsafe { &mut *self.data.get() }
    }
}

impl<'a, T: ?Sized + fmt::Debug, S: Sched> fmt::Debug for SleepMutexGuard<'a, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized, S: Sched> Deref for SleepMutexGuard<'a, T, S> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized, S: Sched> DerefMut for SleepMutexGuard<'a, T, S> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized, S: Sched> Drop for SleepMutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    fn drop(&mut self) {
        let mut inner = self.lock.lock();
        inner.locked = false;
        // Wake up all processes sleeping on chan.
        // Must be called without any thread lock.
        S::wakeup(inner.id);
    }
}
