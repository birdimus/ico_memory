use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// Holds up 2^31 values
pub struct IndexSpinlock<T> {
    lock: AtomicU32,
    data: UnsafeCell<T>,
}

impl<T> IndexSpinlock<T> {
    const LOCK: u32 = 1 << 31;
    const MASK: u32 = !(IndexSpinlock::<T>::LOCK); // | IndexSpinlock::PANIC

    #[inline(always)]
    pub const fn new(value: u32, data: T) -> IndexSpinlock<T> {
        return IndexSpinlock {
            lock: AtomicU32::new(value & IndexSpinlock::<T>::MASK),
            data: UnsafeCell::new(data),
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
    pub fn lock(&self) -> IndexSpinlockGuard<T> {
        #[cfg(any(test, feature = "std"))]
        let mut counter = 0;
        let mut lock_value = self.lock.load(Ordering::Acquire);
        loop {
            if lock_value < IndexSpinlock::<T>::LOCK {
                let target = lock_value | IndexSpinlock::<T>::LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::SeqCst,
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
                    counter = IndexSpinlock::<T>::increment_yield_counter(counter);
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
        return self.lock.load(Ordering::Relaxed) & IndexSpinlock::<T>::MASK;
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    #[inline(always)]
    pub fn set(&mut self, value: u32) {
        return self
            .lock
            .store(value & IndexSpinlock::<T>::MASK, Ordering::Relaxed);
    }

    /// Move the value out of the lock, consuming the lock;
    #[inline(always)]
    pub fn into_inner(self) -> u32 {
        return self.lock.load(Ordering::Relaxed) & IndexSpinlock::<T>::MASK;
    }
}

unsafe impl<T : Send> Send for IndexSpinlock<T> {}
unsafe impl<T : Sync> Sync for IndexSpinlock<T> {}

pub struct IndexSpinlockGuard<'a, T: 'a> {
    lock: &'a IndexSpinlock<T>,
    value: u32,
}



impl<'a, T> IndexSpinlockGuard<'a, T> {
    /// Get the value that was set on the lock at acquisition time.
    #[inline(always)]
    pub fn read(&self) -> u32 {
        return self.value;
    }
    /// Set the value that will be written on release.
    #[inline(always)]
    pub fn write(&mut self, value: u32) {
        return self.value = value & IndexSpinlock::<T>::MASK;
    }
}
impl<'a, T> Drop for IndexSpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.lock.store(self.value, Ordering::Release);
    }
}

impl<'a, T> Deref for IndexSpinlockGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        return unsafe { &*self.lock.data.get() };
    }
}

impl<'a, T> DerefMut for IndexSpinlockGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        return unsafe { &mut *self.lock.data.get() };
    }
}

#[cfg(test)]
mod test;
