use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// Holds up 2^31 values
#[repr(C)]
#[derive(Debug)]
pub struct Spinlock<T> {
    data: UnsafeCell<T>,
    lock: AtomicU32,
}

impl<T> Spinlock<T> {
    const LOCK: u32 = 1 << 31;
    const MASK: u32 = !(Spinlock::<T>::LOCK); // | Spinlock::PANIC

    #[inline(always)]
    pub const fn new(value: u32, data: T) -> Spinlock<T> {
        return Spinlock {
            lock: AtomicU32::new(value & Spinlock::<T>::MASK),
            data: UnsafeCell::new(data),
        };
    }

    #[cfg(any(test, feature = "std"))]
    #[inline(always)]
    fn increment_yield_counter(value: u32) -> u32 {
        if value > 2 {
            std::thread::yield_now();
            return 0;
        }
        return value + 1;
    }

    #[inline(always)]
    pub fn lock(&self) -> SpinlockGuard<T> {
        #[cfg(any(test, feature = "std"))]
        let mut counter = 0;
        let mut lock_value = self.lock.load(Ordering::Acquire);
        loop {
            if lock_value < Spinlock::<T>::LOCK {
                let target = lock_value | Spinlock::<T>::LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::Acquire,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        return SpinlockGuard {
                            lock: self,
                            value: lock_value,
                        }
                    }
                    Err(x) => lock_value = x,
                }
            } else {
                #[cfg(any(test, feature = "std"))]
                {
                    counter = Spinlock::<T>::increment_yield_counter(counter);
                }

                lock_value = self.lock.load(Ordering::Acquire);
            }
            core::sync::atomic::spin_loop_hint();
        }
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    #[inline(always)]
    pub fn get(&mut self) -> u32 {
        return self.lock.load(Ordering::Relaxed) & Spinlock::<T>::MASK;
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    #[inline(always)]
    pub fn set(&mut self, value: u32) {
        return self
            .lock
            .store(value & Spinlock::<T>::MASK, Ordering::Relaxed);
    }

    /// Move the value out of the lock, consuming the lock;
    #[inline(always)]
    pub fn into_inner(self) -> u32 {
        return self.lock.load(Ordering::Relaxed) & Spinlock::<T>::MASK;
    }
}

unsafe impl<T: Send> Send for Spinlock<T> {}
unsafe impl<T: Send> Sync for Spinlock<T> {}
#[derive(Debug)]
pub struct SpinlockGuard<'a, T: 'a> {
    lock: &'a Spinlock<T>,
    value: u32,
}

impl<'a, T> SpinlockGuard<'a, T> {
    /// Get the value that was set on the lock at acquisition time.
    #[inline(always)]
    pub fn read(&self) -> u32 {
        return self.value;
    }
    /// Set the value that will be written on release.
    #[inline(always)]
    pub fn write(&mut self, value: u32) {
        return self.value = value & Spinlock::<T>::MASK;
    }
}
impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(self.value, Ordering::Release);
    }
}

impl<'a, T> Deref for SpinlockGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        return unsafe { &*self.lock.data.get() };
    }
}

impl<'a, T> DerefMut for SpinlockGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        return unsafe { &mut *self.lock.data.get() };
    }
}

/// Holds up 2^31 values
#[derive(Debug)]
pub struct IndexSpinlock {
    lock: AtomicU32,
}

impl IndexSpinlock {
    const LOCK: u32 = 1 << 31;
    const MASK: u32 = !(IndexSpinlock::LOCK); // | IndexSpinlock::PANIC

    #[inline(always)]
    pub const fn new(value: u32) -> IndexSpinlock {
        return IndexSpinlock {
            lock: AtomicU32::new(value & IndexSpinlock::MASK),
        };
    }

    #[cfg(any(test, feature = "std"))]
    #[inline(always)]
    fn increment_yield_counter(value: u32) -> u32 {
        if value > 1 {
            std::thread::yield_now();
            return 0;
        }
        return value + 1;
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Option<IndexSpinlockGuard> {
        let lock_value = self.lock.load(Ordering::Acquire);
        if lock_value < IndexSpinlock::LOCK {
            let target = lock_value | IndexSpinlock::LOCK;

            match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return Some(IndexSpinlockGuard {
                            lock: self,
                            value: lock_value,
                        });
                    },
                    Err(_) =>{},
                };
        }

        return None;
    }

    #[inline(always)]
    pub fn lock(&self) -> IndexSpinlockGuard {
        #[cfg(any(test, feature = "std"))]
        let mut counter = 0;
        let mut lock_value = self.lock.load(Ordering::Acquire);
        loop {
            if lock_value < IndexSpinlock::LOCK {
                let target = lock_value | IndexSpinlock::LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::Acquire,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        return IndexSpinlockGuard {
                            lock: self,
                            value: lock_value,
                        }
                    }
                    Err(x) => lock_value = x,
                }
            } else {
                #[cfg(any(test, feature = "std"))]
                {
                    counter = IndexSpinlock::increment_yield_counter(counter);
                }

                lock_value = self.lock.load(Ordering::Acquire);
            }
            core::sync::atomic::spin_loop_hint();
        }
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    #[inline(always)]
    pub fn get(&mut self) -> u32 {
        return self.lock.load(Ordering::Relaxed) & IndexSpinlock::MASK;
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    #[inline(always)]
    pub fn set(&mut self, value: u32) {
        return self
            .lock
            .store(value & IndexSpinlock::MASK, Ordering::Relaxed);
    }

    /// Move the value out of the lock, consuming the lock;
    #[inline(always)]
    pub fn into_inner(self) -> u32 {
        return self.lock.load(Ordering::Relaxed) & IndexSpinlock::MASK;
    }
}

unsafe impl Send for IndexSpinlock {}
unsafe impl Sync for IndexSpinlock {}

#[derive(Debug)]
pub struct IndexSpinlockGuard<'a> {
    lock: &'a IndexSpinlock,
    value: u32,
}

impl<'a> IndexSpinlockGuard<'a> {
    /// Get the value that was set on the lock at acquisition time.
    #[inline(always)]
    pub fn read(&self) -> u32 {
        return self.value;
    }
    /// Set the value that will be written on release.
    #[inline(always)]
    pub fn write(&mut self, value: u32) {
        return self.value = value & IndexSpinlock::MASK;
    }
}
impl<'a> Drop for IndexSpinlockGuard<'a> {
    fn drop(&mut self) {
        self.lock.lock.store(self.value, Ordering::Release);
    }
}

#[cfg(test)]
mod test;
