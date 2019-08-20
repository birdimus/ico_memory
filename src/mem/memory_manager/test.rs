#[cfg(test)]
mod test {

    use crate::mem::memory_manager::MemoryManager;
    use crate::mem::queue::Swap;
    use crate::sync::index_lock::IndexSpinlock;
    use core::sync::atomic::AtomicUsize;
    use std::time::Instant;
    use core::alloc::{GlobalAlloc, Layout};
    use std::alloc::{alloc, dealloc};

    const MAX_64: usize = 1024 * 2048;
    const MAX_128: usize = 1024 * 1024;
    const MAX_256: usize = 1024 * 512;
    const MAX_512: usize = 1024 * 256;
    const MAX_1024: usize = 1024 * 128;
    const MAX_2048: usize = 1024 * 64;

    static BUFFER_64: [AtomicUsize; MAX_64] =
        unsafe { Swap::<[usize; MAX_64], [AtomicUsize; MAX_64]>::get([0; MAX_64]) };
    static BUFFER_128: [AtomicUsize; MAX_128] =
        unsafe { Swap::<[usize; MAX_128], [AtomicUsize; MAX_128]>::get([0; MAX_128]) };
    static BUFFER_256: [AtomicUsize; MAX_256] =
        unsafe { Swap::<[usize; MAX_256], [AtomicUsize; MAX_256]>::get([0; MAX_256]) };
    static BUFFER_512: [AtomicUsize; MAX_512] =
        unsafe { Swap::<[usize; MAX_512], [AtomicUsize; MAX_512]>::get([0; MAX_512]) };
    static BUFFER_1024: [AtomicUsize; MAX_1024] =
        unsafe { Swap::<[usize; MAX_1024], [AtomicUsize; MAX_1024]>::get([0; MAX_1024]) };
    static BUFFER_2048: [AtomicUsize; MAX_2048] =
        unsafe { Swap::<[usize; MAX_2048], [AtomicUsize; MAX_2048]>::get([0; MAX_2048]) };

    static MANAGER: MemoryManager = MemoryManager::new(
        &BUFFER_64,
        MAX_64,
        &BUFFER_128,
        MAX_128,
        &BUFFER_256,
        MAX_256,
        &BUFFER_512,
        MAX_512,
        &BUFFER_1024,
        MAX_1024,
        &BUFFER_2048,
        MAX_2048,
    );
    static LOCK: IndexSpinlock = IndexSpinlock::new(0);

    #[test]
    fn custom_alloc() {
        let lock = LOCK.lock();
        unsafe{MANAGER.clear();}
        let now = Instant::now();
        let mut cells : Vec<*mut u8> = Vec::with_capacity(2048);
        for j in 0..256{
            
            // letlayout = Layout::from_size_align(64,16).ok().unwrap();
            for i in 0..2048{
                let layout = Layout::from_size_align(i,16).ok().unwrap();
                let mut raw = unsafe{MANAGER.alloc(layout)};

                // println!("raw map {} {} {}", raw as usize, i, j);
                cells.push(raw);
                let size = if layout.size() < 1 {1} else{layout.size()};
                unsafe{raw.write_bytes(i as u8, size );}
                assert_eq!(unsafe{cells[i].read()}, i as u8);
            }

            for i in 0..2048{

                assert_eq!(unsafe{cells[i].read()}, i as u8);
            }

            
            for i in 0..2048{
                let last_val = 2048 - i -1;

                let layout = Layout::from_size_align(last_val,16).ok().unwrap();
                unsafe{MANAGER.dealloc(cells.pop().unwrap(), layout)};
            }
            // unsafe{MANAGER.clear();}
        }
        println!("custom alloc {} micros", now.elapsed().as_micros());
        unsafe{MANAGER.clear();}
    }

     #[test]
    fn default_alloc() {
        let lock = LOCK.lock();
        let now = Instant::now();
        let mut cells : Vec<*mut u8> = Vec::with_capacity(2048);
        for j in 0..256{
            // let layout = Layout::from_size_align(64,16).ok().unwrap();
            for i in 0..2048{
                let layout = Layout::from_size_align(i,16).ok().unwrap();
                let mut raw = unsafe{alloc(layout)};
                 // println!("raw map {} {}", raw as usize, i);
                cells.push(raw);
                let size = if layout.size() < 1 {1} else{layout.size()};
                unsafe{raw.write_bytes(i as u8, size);}
                assert_eq!(unsafe{cells[i].read()}, i as u8);
            }

            for i in 0..2048{

                assert_eq!(unsafe{cells[i].read()}, i as u8);
            }

            
            for i in 0..2048{
                let last_val = 2048 - i -1;
                let layout = Layout::from_size_align(last_val,16).ok().unwrap();
                unsafe{dealloc(cells.pop().unwrap(), layout)};
            }
        }
        println!("default alloc {} micros", now.elapsed().as_micros());

    }
}
