use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

pub struct RWSpinLock<T> {
    lock: AtomicU32,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for RWSpinLock<T> {}
unsafe impl<T: Sync> Sync for RWSpinLock<T> {}

impl<T> RWSpinLock<T> {
    const WRITE_LOCK: u32 = 1 << 31;
    const WRITE_REQUEST: u32 = 1 << 30;
    // const WRITE_MASK : u32 = RWSpinLock::<T>::WRITE_LOCK | RWSpinLock::<T>::WRITE_REQUEST;

    #[inline]
    pub const fn new(data: T) -> RWSpinLock<T> {
        return RWSpinLock {
            lock: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        };
    }
    #[inline]
    pub fn read(&self) -> RWSpinReadGuard<T> {
        let mut lock_value = self.lock.load(Ordering::Acquire);
        loop {
            if lock_value < RWSpinLock::<T>::WRITE_REQUEST {
                let target = lock_value + 1;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::SeqCst,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        return RWSpinReadGuard {
                            lock: self,
                            // data: unsafe { &*self.data.get() },
                        };
                    }
                    Err(x) => lock_value = x,
                }
            } else {
                lock_value = self.lock.load(Ordering::Acquire);
            }
            core::sync::atomic::spin_loop_hint();
        }
    }
    #[inline]
    pub fn write(&self) -> RWSpinWriteGuard<T> {
        let mut lock_value = self
            .lock
            .fetch_or(RWSpinLock::<T>::WRITE_REQUEST, Ordering::SeqCst);
        loop {
            if lock_value == RWSpinLock::<T>::WRITE_REQUEST {
                let target = RWSpinLock::<T>::WRITE_LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return RWSpinWriteGuard {
                            lock: self,
                            // data: unsafe { &mut *self.data.get() },
                        };
                    }
                    Err(_x) => {}
                }
            } else {
                core::sync::atomic::spin_loop_hint();
            }
            // We must continually request, because a write lock will clear all write flags on release
            lock_value = self
                .lock
                .fetch_or(RWSpinLock::<T>::WRITE_REQUEST, Ordering::SeqCst);
        }
    }
    #[inline]
    pub fn upgrade(&self, read: RWSpinReadGuard<T>) -> RWSpinWriteGuard<T> {
        let mut lock_value = self
            .lock
            .fetch_or(RWSpinLock::<T>::WRITE_REQUEST, Ordering::SeqCst);
        core::mem::drop(read);
        loop {
            if lock_value == RWSpinLock::<T>::WRITE_REQUEST {
                let target = RWSpinLock::<T>::WRITE_LOCK;
                match self.lock.compare_exchange_weak(
                    lock_value,
                    target,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return RWSpinWriteGuard {
                            lock: self,
                            // data: unsafe { &mut *self.data.get() },
                        };
                    }
                    Err(_x) => {}
                }
            } else {
                core::sync::atomic::spin_loop_hint();
            }
            // We must continually request, because a write lock will clear all write flags on release
            lock_value = self
                .lock
                .fetch_or(RWSpinLock::<T>::WRITE_REQUEST, Ordering::SeqCst);
        }
    }

    /// Since this call borrows the Lock mutably, no actual locking needs to take place --
    /// the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        return unsafe { &mut *self.data.get() };
    }

    /// Move the value out of the rwlock, consuming the lock;
    pub fn into_inner(self) -> T {
        return self.data.into_inner();
    }
}
pub struct RWSpinReadGuard<'a, T: 'a> {
    lock: &'a RWSpinLock<T>,
}

impl<'a, T> Drop for RWSpinReadGuard<'a, T> {
    fn drop(&mut self) {
        //println!("dropped read");
        self.lock.lock.fetch_sub(1, Ordering::SeqCst);
    }
}
impl<'a, T> Deref for RWSpinReadGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        return unsafe { &*self.lock.data.get() };
    }
}

pub struct RWSpinWriteGuard<'a, T: 'a> {
    lock: &'a RWSpinLock<T>,
}

impl<'a, T> Drop for RWSpinWriteGuard<'a, T> {
    fn drop(&mut self) {
        //println!("dropped write");
        self.lock.lock.store(0, Ordering::SeqCst);
        //self.lock.fetch_and(!RWSpinLock::<T>::WRITE_MASK, Ordering::SeqCst);
    }
}
impl<'a, T> Deref for RWSpinWriteGuard<'a, T> {
    type Target = T;
    fn deref<'b>(&'b self) -> &'b T {
        return unsafe { &*self.lock.data.get() };
    }
}

impl<'a, T> DerefMut for RWSpinWriteGuard<'a, T> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut T {
        return unsafe { &mut *self.lock.data.get() };
    }
}
