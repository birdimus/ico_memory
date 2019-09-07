#[cfg(test)]
mod test {
    use crate::mem::queue::QUEUE32_NULL;
    // use crate::mem::resource_manager::Resource;
    use crate::mem::resource_manager::ResourceManager;
    use crate::sync::index_lock::IndexSpinlock;
    use core::sync::atomic::AtomicU32;

    static mut QUEUE_BUFFER: [u32; 1024] = [QUEUE32_NULL; 1024];

    // unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([QUEUE32_NULL; 1024]) };
    static mut SLOT_BUFFER: [u32; 1024] = [0; 1024];
    // unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([0; 1024]) };
    static mut COUNT_BUFFER: [u32; 1024] = [0; 1024];
    // unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([0; 1024]) };

    //This MUST be mutable.
    static mut RAW_DATA_BUFFER: [u8; 1024 * std::mem::size_of::<Simple>()] =
        [0; 1024 * std::mem::size_of::<Simple>()];

    static MANAGER: ResourceManager<Simple> = unsafe {
        ResourceManager::from_static(
            &SLOT_BUFFER[0] as *const u32 as *mut AtomicU32,
            &COUNT_BUFFER[0] as *const u32 as *mut AtomicU32,
            &QUEUE_BUFFER[0] as *const u32 as *mut AtomicU32,
            &RAW_DATA_BUFFER[0] as *const u8 as *mut Simple,
            1024,
        )
    };
    static LOCK: IndexSpinlock = IndexSpinlock::new(0);

    struct Simple {
        data: u64,
    }

    impl Drop for Simple {
        fn drop(&mut self) {
            // println!("drop {}",self.data);
        }
    }

    #[test]
    fn init() {
        let _l = LOCK.lock();
        for _k in 0..65535 {
            let mut t: Vec<u64> = Vec::new();
            for i in 0..16 {
                t.push(MANAGER.retain(Simple { data: i }).unwrap());
            }
            for i in 0..16 {
                assert_eq!(MANAGER.release(t.pop().unwrap()), true, "{}", i);
            }
        }
    }

    #[test]
    fn retain_clone_release() {
        let _l = LOCK.lock();
        for _k in 0..65535 {
            let mut t: Vec<u64> = Vec::new();
            let mut q: Vec<&Simple> = Vec::new();
            for i in 0..16 {
                let tmp = MANAGER.retain(Simple { data: i }).unwrap();
                t.push(tmp);
                unsafe {
                    q.push(MANAGER.retain_reference(tmp).unwrap());
                }
            }

            for i in 0..16 {
                unsafe {
                    q.push(MANAGER.clone_reference(&q[i]));
                }
            }
            for i in 0..32 {
                assert_eq!(q[i].data, i as u64 % 16);
            }
            for i in 0..16 {
                assert_eq!(MANAGER.release(t.pop().unwrap()), true, "{}", i);
                unsafe {
                    MANAGER.release_reference(q.pop().unwrap());
                    MANAGER.release_reference(q.pop().unwrap());
                }
            }
        }
    }
}
