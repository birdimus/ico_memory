use crate::mem::mmap;
use crate::sync::index_lock::IndexSpinlock;
use core::cell::UnsafeCell;
use crate::collections::queue::Queue;


struct MemoryPool<const BLOCK_SIZE: usize, const BLOCKS_PER_CHUNK: usize, const MAX_CHUNKS: usize> {

	active_chunk_remaining_free: IndexSpinlock<[mmap::MapAlloc; MAX_CHUNKS]>,
	// free_queue : Queue<{BLOCKS_PER_CHUNK * MAX_CHUNKS}>,
}

impl<const BLOCK_SIZE: usize,const BLOCKS_PER_CHUNK: usize, const MAX_CHUNKS: usize>  
MemoryPool<{BLOCK_SIZE}, {BLOCKS_PER_CHUNK}, {MAX_CHUNKS}>  {
	const CHUNK_SIZE: usize = (BLOCKS_PER_CHUNK * BLOCK_SIZE);

	pub fn new() -> MemoryPool<{BLOCK_SIZE}, {BLOCKS_PER_CHUNK}, {MAX_CHUNKS}> {
        return MemoryPool::<{BLOCK_SIZE}, {BLOCKS_PER_CHUNK}, {MAX_CHUNKS}> {
            active_chunk_remaining_free: IndexSpinlock::new(0, unsafe { core::mem::zeroed() }),
        };
    }


    fn get_free_block(&self) -> *mut u8 {
        let mut active_chunk_lock = self.active_chunk_remaining_free.lock();

        let active_chunk_data = active_chunk_lock.read();

        // Decompose the atomic value
        let mut remaining_blocks = active_chunk_data & 4095;
        let mut chunk_count = (active_chunk_data >> 12);

        if remaining_blocks == 0 {
            // Make sure we haven't run out of address space.
            if chunk_count >= (MAX_CHUNKS as u32) {
                core::panic!("Memory Addresss Failed.");
            }

            let mem = mmap::alloc_page_aligned(BLOCKS_PER_CHUNK * BLOCK_SIZE);

            // Allocation failed.  This must abort.
            if mem.is_null() {
            	//process::abort();
                core::panic!("Memory Allocation Failed.");
            }

            (*active_chunk_lock)[chunk_count as usize] = mem;
            remaining_blocks = (BLOCKS_PER_CHUNK as u32);
            chunk_count += 1;
        }
        let new_remaining_blocks = remaining_blocks - 1;

        let address = unsafe {
            (*active_chunk_lock)[(chunk_count - 1) as usize]
                .get_unchecked(new_remaining_blocks as isize)
        };

        active_chunk_lock.write(new_remaining_blocks | (chunk_count << 12));

        return address;
    }

    // #[inline(always)]
    pub fn allocate(&self) -> *mut u8 {
        //dequeue - if dequeue fails

        return self.get_free_block();
    }

    /// This is unsafe, because if you pass back a bad pointer there is no checking.
    #[inline(always)]
    pub unsafe fn deallocate(&self, ptr: *mut u8) {}
}


// /// In order to minimize potential lag on expansion - we alloc, but only add to the queue on free.
// struct MemoryPool64 {
//     active_chunk_remaining_free: IndexSpinlock<[mmap::MapAlloc; 1024]>,
// }

// impl MemoryPool64 {//<const BLOCK_SIZE: usize, const MAX_CHUNKS: usize, const BLOCKS_PER_CHUNK: usize>
//     const MAX_CHUNKS: u32 = 1024;
//     const BLOCK_SIZE: u32 = 64;
//     const BLOCKS_PER_CHUNK: u32 = 2048;
//     const CHUNK_SIZE: usize = (MemoryPool64::BLOCKS_PER_CHUNK * MemoryPool64::BLOCK_SIZE) as usize;

//     pub fn new() -> MemoryPool64 {
//         return MemoryPool64 {
//             // memory_chunks: UnsafeCell::new(unsafe{core::mem::zeroed()}),//unsafe{core::mem::zeroed()},
//             active_chunk_remaining_free: IndexSpinlock::new(0, unsafe { core::mem::zeroed() }),
//         };
//     }

//     fn get_free_block(&self) -> *mut u8 {
//         let mut active_chunk_lock = self.active_chunk_remaining_free.lock();

//         let active_chunk_data = active_chunk_lock.read();

//         // Decompose the atomic value
//         let mut remaining_blocks = active_chunk_data & 4095;
//         let mut chunk_count = active_chunk_data >> 12;

//         if remaining_blocks == 0 {
//             // Make sure we haven't run out of address space.
//             if chunk_count >= MemoryPool64::MAX_CHUNKS {
//                 core::panic!("Memory Addresss Failed.");
//             }

//             let mem = mmap::alloc_page_aligned(MemoryPool64::CHUNK_SIZE);

//             // Allocation failed.  This must panic.
//             if mem.is_null() {
//                 core::panic!("Memory Allocation Failed.");
//             }

//             (*active_chunk_lock)[chunk_count as usize] = mem;
//             remaining_blocks = MemoryPool64::BLOCKS_PER_CHUNK;
//             chunk_count += 1;
//         }
//         let new_remaining_blocks = remaining_blocks - 1;

//         let address = unsafe {
//             (*active_chunk_lock)[(chunk_count - 1) as usize]
//                 .get_unchecked(new_remaining_blocks as isize)
//         };

//         active_chunk_lock.write(new_remaining_blocks | (chunk_count << 12));

//         return address;
//     }

//     // #[inline(always)]
//     pub fn allocate(&self) -> *mut u8 {
//         //dequeue - if dequeue fails

//         return self.get_free_block();
//     }

//     /// This is unsafe, because if you pass back a bad pointer there is no checking.
//     #[inline(always)]
//     pub unsafe fn deallocate(&self, ptr: *mut u8) {}
// }

#[cfg(test)]
mod test;
