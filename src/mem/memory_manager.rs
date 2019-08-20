use crate::mem::memory_pool::MemoryPool;
use crate::mem::mmap;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::AtomicUsize;
struct MemoryManager<'a> {
    pool_64: MemoryPool<'a>,
    pool_128: MemoryPool<'a>,
    pool_256: MemoryPool<'a>,
    pool_512: MemoryPool<'a>,
    pool_1024: MemoryPool<'a>,
    pool_2048: MemoryPool<'a>,
}

impl<'a> MemoryManager<'a> {
    pub const fn new(
        slice_64: &'a [AtomicUsize],
        capacity_64: usize,
        slice_128: &'a [AtomicUsize],
        capacity_128: usize,
        slice_256: &'a [AtomicUsize],
        capacity_256: usize,
        slice_512: &'a [AtomicUsize],
        capacity_512: usize,
        slice_1024: &'a [AtomicUsize],
        capacity_1024: usize,
        slice_2048: &'a [AtomicUsize],
        capacity_2048: usize,
    ) -> MemoryManager<'a> {
        return MemoryManager::<'a> {
            pool_64: MemoryPool::new(64, slice_64, capacity_64),
            pool_128: MemoryPool::new(128, slice_128, capacity_128),
            pool_256: MemoryPool::new(256, slice_256, capacity_256),
            pool_512: MemoryPool::new(512, slice_512, capacity_512),
            pool_1024: MemoryPool::new(1024, slice_1024, capacity_1024),
            pool_2048: MemoryPool::new(2048, slice_2048, capacity_2048),
        };
    }
}

// This function is a super duper bad idea
// impl<'a>  MemoryManager<'a> {
// unsafe fn clear(&self){
//         self.pool_64.clear();
//         self.pool_128.clear();
//         self.pool_256.clear();
//         self.pool_512.clear();
//         self.pool_1024.clear();
//         self.pool_2048.clear();
//     }
// }

unsafe impl<'a> GlobalAlloc for MemoryManager<'a> {



    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // All allocations are aligned at their size boundary - so we just need the greater of the two.

        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater : u32 = (allocation_size as u32 - 1).leading_zeros() + 1;
        // println!("pot_greater {}", pot_greater);
        match pot_greater {
            32 => {
                // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }
            31 => {
                // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }
            30 => {
                 // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }
            29 => {
                 // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }

            28 => {
                 // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }
            27 => {
                 // println!("alloc 64 {}", layout.size());
                return self.pool_64.allocate();
                
            }
            26 => {
                 // println!("alloc 128 {}", layout.size());
                return self.pool_128.allocate();
                
            }

            25 => {
                // println!("alloc 256");
                return self.pool_256.allocate();
                
            }
            24 => {
                // println!("alloc 512");
                return self.pool_512.allocate();
                
            }
            23 => {
                 // println!("alloc 1024");
                return self.pool_1024.allocate();
                
            }
            22 => {
               // println!("alloc 2048 {}", layout.size());
                return self.pool_2048.allocate();
                
            }

            _ => {
                // println!("alloc page");
                let page_aligned_size = mmap::get_page_aligned_size(allocation_size);
                return mmap::alloc_page_aligned(page_aligned_size).memory;
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater : u32 = (allocation_size as u32 - 1).leading_zeros() + 1;

        match pot_greater {

            32 => {
                // println!("free 64 {}", layout.size());
                return self.pool_64.deallocate(ptr);
            }
            31 => {
                return self.pool_64.deallocate(ptr);
            }
            30 => {
                return self.pool_64.deallocate(ptr);
            }
            29 => {
                return self.pool_64.deallocate(ptr);
            }
            28 => {
                return self.pool_64.deallocate(ptr);
            }
            27 => {
                return self.pool_64.deallocate(ptr);
            }

            26 => {
                return self.pool_128.deallocate(ptr);
            }
            25 => {
                return self.pool_256.deallocate(ptr);
            }
            24 => {
                return self.pool_512.deallocate(ptr);
            }
            23 => {
                return self.pool_1024.deallocate(ptr);
            }
            22 => {
                // println!("free 2048 {}", layout.size());
                return self.pool_2048.deallocate(ptr);
            }
            _ => {
                let page_aligned_size = mmap::get_page_aligned_size(allocation_size);
                return mmap::free_page_aligned(ptr, page_aligned_size);
            }
        }
    }
}


#[cfg(test)]
mod test;
