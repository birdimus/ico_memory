use crate::mem::mmap;
use crate::mem::queue::Queue;
use crate::sync::index_lock::Spinlock;
use core::num::NonZeroUsize;
use core::ptr;
use core::sync::atomic::AtomicUsize;

pub const MAX_CHUNKS_POT: usize = 10;
pub const MAX_CHUNKS: usize = 1 << MAX_CHUNKS_POT;

struct BaseMemoryPool {
    active_chunk_remaining_free: Spinlock<[mmap::MapAlloc; 1024]>,
    block_size: usize,
    block_count: usize,
}
impl BaseMemoryPool {
    const MAX_BLOCKS: usize = 65536;
    const BLOCK_MASK: u32 = (BaseMemoryPool::MAX_BLOCKS - 1) as u32;
    const CHUNK_SHIFT: usize = 17;
    const MAX_CHUNKS: usize = 1024;

    const fn new(block_size: usize, block_count: usize) -> BaseMemoryPool {
        // assert!(block_size.is_power_of_two());
        // assert!(block_count.is_power_of_two() && block_count <= BaseMemoryPool::MAX_BLOCKS);
        return BaseMemoryPool {
            block_size: block_size,
            block_count: block_count,
            active_chunk_remaining_free: Spinlock::new(0, [mmap::MapAlloc::null(); 1024]),
        };
    }

    fn get_free_block(&self) -> *mut u8 {
        let mut active_chunk_lock = self.active_chunk_remaining_free.lock();

        let active_chunk_data = active_chunk_lock.read();

        // Decompose the atomic value
        let mut remaining_blocks = active_chunk_data & BaseMemoryPool::BLOCK_MASK;
        let mut chunk_count = (active_chunk_data >> BaseMemoryPool::CHUNK_SHIFT);

        if remaining_blocks == 0 {
            // Make sure we haven't run out of address space.
            if chunk_count >= (BaseMemoryPool::MAX_CHUNKS as u32) {
                core::panic!("Memory Addresss Failed.");
                return ptr::null_mut();
                // //handle_alloc_error
            }
            let page_aligned_size = mmap::get_page_aligned_size(self.block_count * self.block_size);
            // println!("chunks  {} {} {}", chunk_count, remaining_blocks, page_aligned_size);
            let mem = mmap::alloc_page_aligned(page_aligned_size);
            // println!("mem  {}", mem.memory as usize);
            // Allocation failed.  This must abort.
            if mem.is_null() {
                core::panic!("Memory Allocation Failed.");
                return ptr::null_mut();
                //process::abort();
                //
            }

            (*active_chunk_lock)[chunk_count as usize] = mem;
            remaining_blocks = (self.block_count as u32);
            chunk_count += 1;
        }
        let new_remaining_blocks = remaining_blocks - 1;

        let address = unsafe {
            (*active_chunk_lock)[(chunk_count - 1) as usize]
                .get_unchecked((new_remaining_blocks * self.block_size as u32) as isize)
        };

        active_chunk_lock
            .write(new_remaining_blocks | (chunk_count << BaseMemoryPool::CHUNK_SHIFT));

        return address;
    }

    fn clear(&self) {
        unsafe {
            let mut active_chunk_lock = self.active_chunk_remaining_free.lock();
            let active_chunk_data = active_chunk_lock.read();

            // Decompose the atomic value
            let mut chunk_count = (active_chunk_data >> BaseMemoryPool::CHUNK_SHIFT);
            for i in 0..chunk_count {
                mmap::free_page_aligned(
                    (*active_chunk_lock)[i as usize].memory,
                    (*active_chunk_lock)[i as usize].size,
                );
                (*active_chunk_lock)[i as usize] = mmap::MapAlloc::null();
            }
            active_chunk_lock.write(0);
        }
    }
}
unsafe impl Send for BaseMemoryPool {}
unsafe impl Sync for BaseMemoryPool {}

impl Drop for BaseMemoryPool {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct MemoryPool<'a> {
    memory_pool: BaseMemoryPool,
    free_queue: Queue<'a>,
}

const fn is_power_of_two_or_zero(value: usize) -> bool {
    //fails for 0
    return (value & (value - 1)) == 0;
}

impl<'a> MemoryPool<'a> {
    pub const fn new(
        block_size: usize,
        //block_count: usize,
        slice: &'a [AtomicUsize],
        capacity: usize,
    ) -> MemoryPool<'a> {
        // assert!(is_power_of_two_or_zero(block_size));
        // assert!(is_power_of_two_or_zero(capacity));
        // assert!(block_size != 0);
        // assert!(capacity >= MAX_CHUNKS);
        return MemoryPool {
            memory_pool: BaseMemoryPool::new(block_size, capacity >> MAX_CHUNKS_POT),
            free_queue: Queue::new(slice, capacity),
        };
    }

    // #[inline(always)]
    pub fn allocate(&self) -> *mut u8 {
        //dequeue - if dequeue fails
        let result = self.free_queue.dequeue();
        match result {
            Some(x) => {
                // println!("dequeue {} {}",x.get(), self.memory_pool.block_size);
                return x.get() as *mut u8;
            }
            None => {
                return self.memory_pool.get_free_block();
            }
        }
    }

    /// This is unsafe, because if you pass back a bad pointer there is no checking.
    #[inline(always)]
    pub unsafe fn deallocate(&self, ptr: *mut u8) {
        // println!("enqueue {} {}", ptr as usize, self.memory_pool.block_size);
        self.free_queue
            .enqueue(NonZeroUsize::new(ptr as usize).unwrap());
    }

    pub unsafe fn clear(&self) {
        self.free_queue.clear();
        self.memory_pool.clear();
    }
}

#[cfg(test)]
mod test;
