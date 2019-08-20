use crate::mem::memory_pool::MemoryPool;
use crate::mem::mmap;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::AtomicUsize;
use core::arch::x86_64::*;

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
impl<'a>  MemoryManager<'a> {
// unsafe fn clear(&self){
//         self.pool_64.clear();
//         self.pool_128.clear();
//         self.pool_256.clear();
//         self.pool_512.clear();
//         self.pool_1024.clear();
//         self.pool_2048.clear();
//     }
    #[inline(always)]
    unsafe fn free_pot(&self, ptr : *mut u8, allocation_size: usize, pot_greater : u32) {
        match pot_greater {
            32 => {
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
                return self.pool_2048.deallocate(ptr);
            }
            _ => {
                let page_aligned_size = mmap::get_page_aligned_size(allocation_size);
                return mmap::free_page_aligned(ptr, page_aligned_size);
            }
        }
    }

    #[inline(always)]
    unsafe fn alloc_pot(&self, allocation_size: usize, pot_greater : u32) -> *mut u8 {
        match pot_greater {
            32 => {
                return self.pool_64.allocate();
            }
            31 => {
                return self.pool_64.allocate();
            }
            30 => {
                return self.pool_64.allocate();
            }
            29 => {
                return self.pool_64.allocate();
            }
            28 => {
                return self.pool_64.allocate();
            }
            27 => {
                return self.pool_64.allocate();
            }
            26 => {
                return self.pool_128.allocate();
            }
            25 => {
                return self.pool_256.allocate();
            }
            24 => {
                return self.pool_512.allocate();
            }
            23 => {
                return self.pool_1024.allocate();
            }
            22 => {
                return self.pool_2048.allocate();
            }
            _ => {
                let page_aligned_size = mmap::get_page_aligned_size(allocation_size);
                return mmap::alloc_page_aligned(page_aligned_size).memory;
            }
        }
    }
}

unsafe impl<'a> GlobalAlloc for MemoryManager<'a> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // All allocations are aligned at their size boundary - so we just need the greater of the two.

        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater: u32 = (allocation_size as u32 - 1).leading_zeros() + 1;
        // println!("pot_greater {}", pot_greater);
        return self.alloc_pot(allocation_size, pot_greater);
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater: u32 = (allocation_size as u32 - 1).leading_zeros() + 1;
        self.free_pot(ptr, allocation_size, pot_greater);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_alloc_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let new_alloc_size = if new_size >= layout.align() {
            new_size
        } else {
            layout.align()
        };

        let pot_old: u32 = (old_alloc_size as u32 - 1).leading_zeros() + 1;
        let pot_new: u32 = (new_alloc_size as u32 - 1).leading_zeros() + 1;
        // If the result is the same allocation size, return the old pointer.
        if pot_old == pot_new || (pot_old < 26 && pot_new < 26) {
            return ptr;
        }


        // Take the minimum size divided by 32
        let old_size = (((layout.size() - 1)>>5)+1) as isize;
        let new_size = (((new_size - 1)>>5)+1)  as isize;

        let copy_size_32 = (if old_size < new_size {old_size} else{new_size});

        let mut new = self.alloc_pot(new_alloc_size, pot_new);

        // AVX memcpy.  We know the memory is aligned to 64 bytes!
        let src = ptr as *const __m256i;
        let dst = new as *mut __m256i;
        for i in 0..copy_size_32{
            _mm256_store_si256(dst, _mm256_load_si256(src.offset(i)));
        }
        
        self.free_pot(ptr, old_alloc_size, pot_old);


        return new;
    }
}

#[cfg(test)]
mod test;
