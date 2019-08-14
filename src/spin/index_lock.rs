use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// Holds up 2^31 values
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

    #[cfg(any(test, feature = "use-std"))]
    #[inline(always)]
    fn increment_yield_counter(value: u32) -> u32 {
        if value > 1 {
            std::thread::yield_now();
            return 0;
        }
        return value + 1;
    }

    #[inline(always)]
    pub fn lock(&self) -> IndexSpinlockGuard {
        #[cfg(any(test, feature = "use-std"))]
        let mut counter = 0;
        let mut lock_value = self.lock.load(Ordering::Acquire);
        loop {
            if lock_value < IndexSpinlock::LOCK {
                let target = lock_value | IndexSpinlock::LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::SeqCst,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        return IndexSpinlockGuard {
                            lock: &self.lock,
                            value: lock_value,
                        }
                    }
                    Err(x) => lock_value = x,
                }
            } else {
                #[cfg(any(test, feature = "use-std"))]
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
pub struct IndexSpinlockGuard<'a> {
    lock: &'a AtomicU32,
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
        self.lock.store(self.value, Ordering::Release);
    }
}
