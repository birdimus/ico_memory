use crate::mem::mmap;
use crate::sync::index_lock::IndexSpinlock;
use core::cell::UnsafeCell;
use crate::mem::queue::Queue;
use core::num::NonZeroUsize;
use core::ptr;


const QUEUE_SIZE : usize = 1024;

struct MemoryPool{

	active_chunk_remaining_free: IndexSpinlock<[mmap::MapAlloc; 1024]>,
	block_size : usize,
	block_count : usize,
	free_queue : Queue<{QUEUE_SIZE}>,
}

impl MemoryPool {

	const MAX_BLOCKS: usize = 65536;
	const BLOCK_MASK: u32 = (MemoryPool::MAX_BLOCKS - 1) as u32;
	const CHUNK_SHIFT: usize = 17;
	const MAX_CHUNKS: usize = 1024;
	//const CHUNK_SIZE: usize = (BLOCKS_PER_CHUNK * BLOCK_SIZE);

	pub fn new(block_size : usize, block_count : usize) -> MemoryPool{
		assert!(block_size.is_power_of_two());
		assert!(block_count.is_power_of_two() && block_count <= MemoryPool::MAX_BLOCKS);
        return MemoryPool{
        	block_size : block_size,
        	block_count : block_count,
            active_chunk_remaining_free: IndexSpinlock::new(0, unsafe { core::mem::zeroed() }),
            free_queue : Queue::new(block_count * MemoryPool::MAX_CHUNKS ),
        };
    }


    fn get_free_block(&self) -> *mut u8 {
        let mut active_chunk_lock = self.active_chunk_remaining_free.lock();

        let active_chunk_data = active_chunk_lock.read();

        // Decompose the atomic value
        let mut remaining_blocks = active_chunk_data & MemoryPool::BLOCK_MASK;
        let mut chunk_count = (active_chunk_data >> MemoryPool::CHUNK_SHIFT);

        if remaining_blocks == 0 {
            // Make sure we haven't run out of address space.
            if chunk_count >= (MemoryPool::MAX_CHUNKS as u32) {
            	return ptr::null_mut();
                //core::panic!("Memory Addresss Failed."); //handle_alloc_error
            }

            let mem = mmap::alloc_page_aligned(self.block_count * self.block_size);

            // Allocation failed.  This must abort.
            if mem.is_null() {
            	return ptr::null_mut();
            	//process::abort();
                //core::panic!("Memory Allocation Failed.");
            }

            (*active_chunk_lock)[chunk_count as usize] = mem;
            remaining_blocks = (self.block_count as u32);
            chunk_count += 1;
        }
        let new_remaining_blocks = remaining_blocks - 1;

        let address = unsafe {
            (*active_chunk_lock)[(chunk_count - 1) as usize]
                .get_unchecked(new_remaining_blocks as isize)
        };

        active_chunk_lock.write(new_remaining_blocks | (chunk_count << MemoryPool::CHUNK_SHIFT));

        return address;
    }

    // #[inline(always)]
    pub fn allocate(&self) -> *mut u8 {
        //dequeue - if dequeue fails
        let result = self.free_queue.dequeue();
        match result{
        	Some(x)=> {return x.get() as *mut u8;},
        	None=> {return self.get_free_block();},
        }
        
    }

    /// This is unsafe, because if you pass back a bad pointer there is no checking.
    #[inline(always)]
    pub unsafe fn deallocate(&self, ptr: *mut u8) {
    	self.free_queue.enqueue(NonZeroUsize::new(ptr as usize).unwrap());
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
    	unsafe{
    		let mut active_chunk_lock = self.active_chunk_remaining_free.lock();
    		 let active_chunk_data = active_chunk_lock.read();

	        // Decompose the atomic value
	        let mut chunk_count = (active_chunk_data >> MemoryPool::CHUNK_SHIFT);
    		for i in 0..chunk_count{
    			mmap::free_page_aligned((*active_chunk_lock)[i as usize].memory, (*active_chunk_lock)[i as usize].size);
    		}	
    		
    	}
	}
}


#[cfg(test)]
mod test;
