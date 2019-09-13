#[cfg(test)]
mod test {

    use crate::mem::MemoryManager;
    // use crate::mem::queue::Swap;
    use crate::sync::IndexSpinlock;
    use core::alloc::{GlobalAlloc, Layout};
    use core::sync::atomic::AtomicUsize;
    use std::alloc::{alloc_zeroed, dealloc, realloc};
    use std::time::Instant;

    const MAX_64: usize = 1024 * 2048;
    const MAX_128: usize = 1024 * 1024;
    const MAX_256: usize = 1024 * 512;
    const MAX_512: usize = 1024 * 256;
    const MAX_1024: usize = 1024 * 128;
    const MAX_2048: usize = 1024 * 64;

    static mut BUFFER_64: [usize; MAX_64] = [0; MAX_64];
    static mut BUFFER_64_PTR : *mut AtomicUsize = unsafe{&BUFFER_64[0] as *const usize as *mut AtomicUsize};
    static mut BUFFER_128: [usize; MAX_128] = [0; MAX_128];
    static mut BUFFER_128_PTR : *mut AtomicUsize = unsafe{&BUFFER_128[0] as *const usize as *mut AtomicUsize};
    static mut BUFFER_256: [usize; MAX_256] = [0; MAX_256];
    static mut BUFFER_256_PTR : *mut AtomicUsize = unsafe{&BUFFER_256[0] as *const usize as *mut AtomicUsize};
    static mut BUFFER_512: [usize; MAX_512] = [0; MAX_512];
    static mut BUFFER_512_PTR : *mut AtomicUsize = unsafe{&BUFFER_512[0] as *const usize as *mut AtomicUsize};
    static mut BUFFER_1024: [usize; MAX_1024] = [0; MAX_1024];
    static mut BUFFER_1024_PTR : *mut AtomicUsize = unsafe{&BUFFER_1024[0] as *const usize as *mut AtomicUsize};
    static mut BUFFER_2048: [usize; MAX_2048] = [0; MAX_2048];
    static mut BUFFER_2048_PTR : *mut AtomicUsize = unsafe{&BUFFER_2048[0] as *const usize as *mut AtomicUsize};

    // Note: as a comparison, one can mark this as the global allocator
    // #[global_allocator]
    static MANAGER: MemoryManager = unsafe {
        MemoryManager::from_static(
            &BUFFER_64_PTR,
            MAX_64,
            &BUFFER_128_PTR,
            MAX_128,
            &BUFFER_256_PTR,
            MAX_256,
            &BUFFER_512_PTR,
            MAX_512,
            &BUFFER_1024_PTR,
            MAX_1024,
            &BUFFER_2048_PTR,
            MAX_2048,
        )
    };
    static LOCK: IndexSpinlock = IndexSpinlock::new(0);

    #[test]
    fn custom_alloc() {
        let _lock = LOCK.lock();
        // unsafe{MANAGER.clear();}
        let now = Instant::now();
        let alloc_count = 256;
        let mut cells: Vec<*mut u8> = Vec::with_capacity(alloc_count);
        for _j in 0..2048 {
            // letlayout = Layout::from_size_align(64,16).ok().unwrap();
            for i in 0..alloc_count {
                let layout = Layout::from_size_align(i + 1, 16).ok().unwrap();
                let raw = unsafe { MANAGER.alloc_zeroed(layout) };

                // println!("raw map {} {} {}", raw as usize, i, j);
                cells.push(raw);
                let size = 1; //if layout.size() < 1 {1} else{layout.size()};
                unsafe {
                    raw.write_bytes(i as u8, size);
                }
                assert_eq!(unsafe { cells[i].read() }, i as u8);
            }

            for i in 0..alloc_count {
                assert_eq!(unsafe { cells[i].read() }, i as u8);
            }

            for i in 0..alloc_count {
                let last_val = alloc_count - i - 1;

                let layout = Layout::from_size_align(last_val + 1, 16).ok().unwrap();
                unsafe { MANAGER.dealloc(cells.pop().unwrap(), layout) };
            }
            // unsafe{MANAGER.clear();}
        }
        println!("custom alloc {} micros", now.elapsed().as_micros());
        // unsafe{MANAGER.clear();}
    }

    #[test]
    fn default_alloc() {
        let _lock = LOCK.lock();
        let now = Instant::now();
        let alloc_count = 256;
        let mut cells: Vec<*mut u8> = Vec::with_capacity(alloc_count);
        for _j in 0..2048 {
            // let layout = Layout::from_size_align(64,16).ok().unwrap();
            for i in 0..alloc_count {
                let layout = Layout::from_size_align(i + 1, 16).ok().unwrap();
                let raw = unsafe { alloc_zeroed(layout) };
                // println!("raw map {} {}", raw as usize, i);
                cells.push(raw);
                let size = 1; //if layout.size() < 1 {1} else{layout.size()};
                unsafe {
                    raw.write_bytes(i as u8, size);
                }
                assert_eq!(unsafe { cells[i].read() }, i as u8);
            }

            for i in 0..alloc_count {
                assert_eq!(unsafe { cells[i].read() }, i as u8);
            }

            for i in 0..alloc_count {
                let last_val = alloc_count - i - 1;
                let layout = Layout::from_size_align(last_val + 1, 16).ok().unwrap();
                unsafe { dealloc(cells.pop().unwrap(), layout) };
            }
        }
        println!("default alloc {} micros", now.elapsed().as_micros());
    }

    #[test]
    fn custom_realloc() {
        let _lock = LOCK.lock();
        // unsafe{MANAGER.clear();}
        let now = Instant::now();
        let alloc_count = 256;
        // let mut cells: Vec<*mut u8> = Vec::with_capacity(alloc_count);
        for _j in 0..2048 {
            let mut layout = Layout::from_size_align(1, 16).ok().unwrap();
            let mut raw = unsafe { MANAGER.alloc_zeroed(layout) };
            unsafe {
                raw.write_bytes(0 as u8, 1);
            }
            for i in 0..alloc_count {
                raw = unsafe { MANAGER.realloc(raw, layout, i + 1) };
                layout = Layout::from_size_align(i + 1, 16).ok().unwrap();
                // println!("raw map {} {} {}", raw as usize, i, j);
                assert_ne!(raw, core::ptr::null_mut());
                // let size = 1; //if layout.size() < 1 {1} else{layout.size()};
                //Validate the copy.

                unsafe {
                    // println!("write {}",i+1);
                    raw.write_bytes((i + 1) as u8, 1);
                }
                // assert_eq!(unsafe { cells[i].read() }, i as u8);
            }
            unsafe { MANAGER.dealloc(raw, layout) };

            // unsafe{MANAGER.clear();}
        }
        println!("custom realloc {} micros", now.elapsed().as_micros());
        // unsafe{MANAGER.clear();}
    }
    #[test]
    fn default_realloc() {
        let _lock = LOCK.lock();
        // unsafe{MANAGER.clear();}
        let now = Instant::now();
        let alloc_count = 256;
        // let mut cells: Vec<*mut u8> = Vec::with_capacity(alloc_count);
        for _j in 0..2048 {
            let mut layout = Layout::from_size_align(1, 16).ok().unwrap();
            let mut raw = unsafe { alloc_zeroed(layout) };
            unsafe {
                raw.write_bytes(0 as u8, 1);
            }
            for i in 0..alloc_count {
                raw = unsafe { realloc(raw, layout, i + 1) };
                layout = Layout::from_size_align(i + 1, 16).ok().unwrap();
                // println!("raw map {} {} {}", raw as usize, i, j);
                assert_ne!(raw, core::ptr::null_mut());
                // let size = 1; //if layout.size() < 1 {1} else{layout.size()};
                //Validate the copy.

                unsafe {
                    // println!("write {}",i+1);
                    raw.write_bytes((i + 1) as u8, 1);
                }
                // assert_eq!(unsafe { cells[i].read() }, i as u8);
            }
            unsafe { dealloc(raw, layout) };

            // unsafe{MANAGER.clear();}
        }
        println!("default realloc {} micros", now.elapsed().as_micros());
        // unsafe{MANAGER.clear();}
    }
    #[test]
    fn custom_realloc_copy() {
        let _lock = LOCK.lock();
        // unsafe{MANAGER.clear();}
        // let now = Instant::now();
        let alloc_count = 256;
        // let mut cells: Vec<*mut u8> = Vec::with_capacity(alloc_count);
        for _j in 0..2048 {
            let mut layout = Layout::from_size_align(1, 16).ok().unwrap();
            let mut raw = unsafe { MANAGER.alloc_zeroed(layout) };
            unsafe {
                raw.write_bytes(0 as u8, 1);
            }
            for i in 0..alloc_count {
                raw = unsafe { MANAGER.realloc(raw, layout, i + 1) };
                layout = Layout::from_size_align(i + 1, 16).ok().unwrap();
                // println!("raw map {} {} {}", raw as usize, i, j);
                assert_ne!(raw, core::ptr::null_mut());
                // let size = 1; //if layout.size() < 1 {1} else{layout.size()};
                //Validate the copy.
                for k in 0..i {
                    unsafe {
                        assert_eq!(raw.offset(k as isize).read(), (i) as u8);
                    }
                }
                unsafe {
                    // println!("write {}",i+1);
                    raw.write_bytes((i + 1) as u8, i + 1);
                }
                // assert_eq!(unsafe { cells[i].read() }, i as u8);
            }
            unsafe { MANAGER.dealloc(raw, layout) };

            // unsafe{MANAGER.clear();}
        }
        // println!("custom realloc {} micros", now.elapsed().as_micros());
        // unsafe{MANAGER.clear();}
    }
}
