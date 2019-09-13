#[cfg(test)]
mod test {

    use crate::mem::memory_pool::MemoryPool;
    // use crate::mem::queue::Swap;
    // use crate::sync::index_lock::IndexSpinlock;
    use core::sync::atomic::AtomicUsize;

    // static BUFFER: [AtomicUsize; 1024 * 2048] =
    // unsafe { Swap::<[usize; 1024 * 2048], [AtomicUsize; 1024 * 2048]>::get([0; 1024 * 2048]) };
    // static POOL: MemoryPool = MemoryPool::new(64, &BUFFER, 1024 * 2048);
    // static LOCK: IndexSpinlock = IndexSpinlock::new(0);

    #[test]
    fn alloc() {
        unsafe {
            let mut buffer_local: [usize; 4096] = [0; 4096];
            let buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
            // unsafe { Swap::<[usize; 4096], [AtomicUsize; 4096]>::get([0; 4096]) };
            let mp = MemoryPool::from_static(64, &buffer_ptr, 4096);
            for _i in 0..4096 {
                assert_ne!(mp.allocate(), core::ptr::null_mut());
            }
        }
        // assert_eq!(mp.allocate(), core::ptr::null_mut());
    }

    #[test]
    fn alloc_free() {
        unsafe {
            let mut buffer_local: [usize; 4096] = [0; 4096];
            let buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
            // unsafe { Swap::<[usize; 4096], [AtomicUsize; 4096]>::get([0; 4096]) };
            let mp = MemoryPool::from_static(64, &buffer_ptr, 4096);

            let mut storage: [*mut u8; 4096] = [core::ptr::null_mut(); 4096];
            for i in 0..4096 {
                storage[i] = mp.allocate();
                assert_ne!(storage[i], core::ptr::null_mut());
            }
            // assert_eq!(mp.allocate(), core::ptr::null_mut());

            for i in 0..4096 {
                mp.deallocate(storage[i]);
            }
            for i in 0..4096 {
                storage[i] = mp.allocate();
                assert_ne!(storage[i], core::ptr::null_mut());
            }
        }
    }
}
