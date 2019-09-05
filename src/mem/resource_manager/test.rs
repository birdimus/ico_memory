

#[cfg(test)]
mod test {
    use crate::mem::queue::QUEUE32_NULL;
    use crate::mem::resource_manager::ResourceManager;
    use crate::mem::queue::Swap;
    use crate::sync::index_lock::IndexSpinlock;
    use core::sync::atomic::AtomicU32;
    use std::mem::MaybeUninit;

static QUEUE_BUFFER: [AtomicU32; 1024] =
        unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([QUEUE32_NULL; 1024]) };
static SLOT_BUFFER: [AtomicU32; 1024] =
        unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([0; 1024]) };
static COUNT_BUFFER: [AtomicU32; 1024] =
        unsafe { Swap::<[u32; 1024], [AtomicU32; 1024]>::get([0; 1024]) };

//This MUST be mutable.
static mut DATA_BUFFER: [MaybeUninit<Simple>; 1024] = unsafe { 
    Swap::<[u64; 1024], [MaybeUninit<Simple>; 1024]>::get([0; 1024]) };

static MANAGER : ResourceManager<Simple> = unsafe{ResourceManager::new(&SLOT_BUFFER,&COUNT_BUFFER, &QUEUE_BUFFER,&DATA_BUFFER, 1024 )};
static LOCK: IndexSpinlock = IndexSpinlock::new(0);

struct Simple{
    data:u64,
}

impl Drop for Simple{
     fn drop(&mut self){
        // println!("drop {}",self.data);
    }
}



#[test]
fn init() {
    let l = LOCK.lock();
    for k in 0..65535{
        let mut t : Vec<u64> = Vec::new();
        for i in 0..16{
            t.push(MANAGER.retain(Simple{data:i}).unwrap());
        }
        for i in 0..16{
            assert_eq!(MANAGER.release(t.pop().unwrap()), true, "{}", i);
        }
        
    }

}

#[test]
fn retain_clone_release() {
    let l = LOCK.lock();
    for k in 0..65535{
        let mut t : Vec<u64> = Vec::new();
        let mut q : Vec<&Simple> = Vec::new();
        for i in 0..16{
            let tmp = MANAGER.retain(Simple{data:i}).unwrap();
            t.push(tmp);
            unsafe{
                q.push(MANAGER.retain_reference(tmp).unwrap());
            }
        }

        for i in 0..16{
            unsafe{
                
            q.push(MANAGER.clone_reference(q[i]));
            }
        }
        for i in 0..32{
            assert_eq!(q[i].data, i as u64 %16);
        }
        for i in 0..16{

            assert_eq!(MANAGER.release(t.pop().unwrap()), true, "{}", i);
            unsafe{
                MANAGER.release_reference(q.pop().unwrap());
                MANAGER.release_reference(q.pop().unwrap());
            }
        }
        
    }

}



}