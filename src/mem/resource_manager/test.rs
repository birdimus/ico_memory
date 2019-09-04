

#[cfg(test)]
mod test {
    use crate::mem::queue::QUEUE32_NULL;
    use crate::mem::resource_manager::ResourceManager;
    use crate::mem::resource_manager::Handle;
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
static mut DATA_BUFFER: [MaybeUninit<u64>; 1024] = [MaybeUninit::new(0);1024];

static MANAGER : ResourceManager<u64> = unsafe{ResourceManager::new(&SLOT_BUFFER,&COUNT_BUFFER, &QUEUE_BUFFER,&DATA_BUFFER, 1024 )};
static LOCK: IndexSpinlock = IndexSpinlock::new(0);

#[test]
fn init() {
    let l = LOCK.lock();
    for k in 0..65535{
        let mut t : Vec<Handle> = Vec::new();
        for i in 0..16{
            t.push(MANAGER.retain(i).unwrap());
        }
        for i in 0..16{
            assert_eq!(MANAGER.release(t.pop().unwrap()), true, "{} {}", i, k);
        }
        
    }
    // assert_eq!(MANAGER.retain(0).is_some(),false) ;

}



}