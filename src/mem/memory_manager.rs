use crate::mem::memory_pool::MemoryPool;
use crate::mem::mmap;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::x86_64::*;
use core::sync::atomic::AtomicUsize;

pub struct MemoryManager {
    pool_64: MemoryPool,
    pool_128: MemoryPool,
    pool_256: MemoryPool,
    pool_512: MemoryPool,
    pool_1024: MemoryPool,
    pool_2048: MemoryPool,
}

impl MemoryManager {
    pub const unsafe fn from_static(
        slice_64: *mut AtomicUsize,
        capacity_64: usize,
        slice_128: *mut AtomicUsize,
        capacity_128: usize,
        slice_256: *mut AtomicUsize,
        capacity_256: usize,
        slice_512: *mut AtomicUsize,
        capacity_512: usize,
        slice_1024: *mut AtomicUsize,
        capacity_1024: usize,
        slice_2048: *mut AtomicUsize,
        capacity_2048: usize,
    ) -> MemoryManager {
        return MemoryManager {
            pool_64: MemoryPool::from_static(64, slice_64, capacity_64),
            pool_128: MemoryPool::from_static(128, slice_128, capacity_128),
            pool_256: MemoryPool::from_static(256, slice_256, capacity_256),
            pool_512: MemoryPool::from_static(512, slice_512, capacity_512),
            pool_1024: MemoryPool::from_static(1024, slice_1024, capacity_1024),
            pool_2048: MemoryPool::from_static(2048, slice_2048, capacity_2048),
        };
    }
}

// This function is a super duper bad idea
impl MemoryManager {
    // unsafe fn clear(&self){
    //         self.pool_64.clear();
    //         self.pool_128.clear();
    //         self.pool_256.clear();
    //         self.pool_512.clear();
    //         self.pool_1024.clear();
    //         self.pool_2048.clear();
    //     }
    #[inline(always)]
    unsafe fn free_pot(&self, ptr: *mut u8, allocation_size: usize, pot_greater: u32) {
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
    unsafe fn alloc_pot(&self, allocation_size: usize, pot_greater: u32) -> *mut u8 {
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

unsafe impl GlobalAlloc for MemoryManager {
    #[inline(always)]
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

    ///  SSE zero using __m128.  Beats rust and naive for loop.
    #[inline(always)]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // MMAP will return zeroed
        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater: u32 = (allocation_size as u32 - 1).leading_zeros() + 1;
        // MMAP will always return zeroed memory - so let's not re-zero it.
        if pot_greater > 22 {
            return self.alloc_pot(allocation_size, pot_greater);
        }

        let new = self.alloc_pot(allocation_size, pot_greater);
        let mut dst = new as *mut __m128i;
        let mut s = layout.size() as isize;
        while s > 0 {
            _mm_store_si128(dst, _mm_setzero_si128());
            dst = dst.offset(1);
            s = s - 16;
        }
        return new;
    }

    #[inline(always)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let allocation_size = if layout.size() >= layout.align() {
            layout.size()
        } else {
            layout.align()
        };

        let pot_greater: u32 = (allocation_size as u32 - 1).leading_zeros() + 1;
        self.free_pot(ptr, allocation_size, pot_greater);
    }
    #[inline(always)]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // println!("ptr {}",ptr as usize);
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
        // println!("pot realloc {} {} {} {}", old_alloc_size, new_alloc_size, pot_old, pot_new);
        // If the result is the same allocation size, return the old pointer.
        if pot_old == pot_new || (pot_old > 26 && pot_new > 26) {
            return ptr;
        }

        let new = self.alloc_pot(new_alloc_size, pot_new);

        {
            //Copy from old to new
            let mut src = ptr as *const __m128i;
            let mut dst = new as *mut __m128i;
            let mut copy_size = if layout.size() < new_size {
                layout.size() as isize
            } else {
                new_size as isize
            };
            while copy_size > 0 {
                _mm_store_si128(dst, _mm_load_si128(src));
                src = src.offset(1);
                dst = dst.offset(1);
                copy_size = copy_size - 16;
            }
        }
        // println!("ptr2 {}",ptr as usize);
        self.free_pot(ptr, old_alloc_size, pot_old);

        return new;
    }
}

#[cfg(test)]
mod test;
