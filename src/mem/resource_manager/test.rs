#[cfg(test)]
mod test {
    use crate::mem::QUEUE_U32_NULL;
    // use crate::mem::resource_manager::Resource;
    use crate::mem::ResourceData;
    use crate::mem::ResourceHandle;
    use crate::mem::ResourceManager;
    use crate::mem::ResourceRef;
    use crate::sync::IndexSpinlock;
    use core::sync::atomic::AtomicU32;
    use std::thread;

    static mut QUEUE_BUFFER: [u32; 1024] = [QUEUE_U32_NULL; 1024];
    static mut QUEUE_PTR: *mut AtomicU32 =
        unsafe { &QUEUE_BUFFER[0] as *const u32 as *mut AtomicU32 };

    static mut RAW_DATA_BUFFER: [u8; 1024 * core::mem::size_of::<ResourceData<Simple>>()] =
        [0; 1024 * core::mem::size_of::<ResourceData<Simple>>()];
    static mut RAW_DATA_BUFFER_PTR: *mut ResourceData<Simple> =
        unsafe { &RAW_DATA_BUFFER[0] as *const u8 as *mut u8 as *mut ResourceData<Simple> };

    static MANAGER: ResourceManager<Simple> =
        unsafe { ResourceManager::from_static(&QUEUE_PTR, &RAW_DATA_BUFFER_PTR, 1024) };
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
        let  _l = LOCK.lock();
        for _k in 0..65535 {
            let mut t: Vec<ResourceHandle> = Vec::new();
            for i in 0..16 {
                t.push(MANAGER.store(Simple { data: i }));
            }
            for i in 0..16 {
                assert_eq!(MANAGER.free(t.pop().unwrap()), true, "{}", i);
            }
            // l.write(k);
        }

    }

    #[test]
    fn retain_clone_release() {
        let  _l = LOCK.lock();
        for _k in 0..65535 {
            let mut t: Vec<ResourceHandle> = Vec::new();
            let mut q: Vec<ResourceRef<Simple>> = Vec::new();
            for i in 0..16 {
                let tmp = MANAGER.store(Simple { data: i });
                t.push(tmp);

                q.push(MANAGER.retain(tmp).unwrap());
            }

            for i in 0..16 {
                q.push(MANAGER.clone(&q[i]));
            }
            for i in 0..32 {
                assert_eq!(MANAGER.get(&q[i]).data, i as u64 % 16);
            }
            for i in 0..16 {
                assert_eq!(MANAGER.free(t.pop().unwrap()), true, "{}", i);

                MANAGER.release(q.pop().unwrap());
                MANAGER.release(q.pop().unwrap());
            }
            // l.write(k);
        }
    }

    #[test]
    fn mt_resources() {
        let  _l = LOCK.lock();
        for _k in 0..4 {
            // let mut t = m.clone();
            // Spin up another thread
            let mut children = vec![];
            children.push(thread::spawn(|| {
                for _j in 0..256 {
                    let mut t: Vec<ResourceHandle> = Vec::new();
                    let mut q: Vec<ResourceRef<Simple>> = Vec::new();
                    for i in 0..256 {
                        let tmp = MANAGER.store(Simple { data: i });
                        t.push(tmp);
                        let r = MANAGER.retain(tmp);
                        assert_eq!(r.is_some(),true );
                        q.push(r.unwrap());
                    }

                    for i in 0..256 {
                        q.push(MANAGER.clone(&q[i]));
                    }
                    for i in 0..512 {
                        assert_eq!(MANAGER.get(&q[i]).data, i as u64 % 256);
                    }
                    for i in 0..256 {
                        let r = t.pop();
                        assert_eq!(r.is_some(),true );
                        assert_eq!(MANAGER.free(r.unwrap()), true, "{}", i);

                        MANAGER.release(q.pop().unwrap());
                        MANAGER.release(q.pop().unwrap());
                    }
                }
            }));
             for child in children {
                // Wait for the thread to finish. Returns a result.
                let _ = child.join();
            }
        }
        // l.write(12);
    }
}
