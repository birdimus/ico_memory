use core::cell::UnsafeCell;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicUsize;
use core::num::NonZeroUsize;
use core::sync::atomic::Ordering;
use crate::sync::index_lock::IndexSpinlock;
use crate::mem::mmap;

use core::marker::PhantomData;
use core::ops::Deref;
use core::mem;
use core::ptr;
struct Unique<T> {
    ptr: *const T,              // *const for variance
    _marker: PhantomData<T>,    // For the drop checker
}

// Deriving Send and Sync is safe because we are the Unique owners
// of this data. It's like Unique<T> is "just" T.
unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    pub fn new(ptr: *mut T) -> Self {
        return Unique { ptr: ptr, _marker: PhantomData };
    }

    pub fn as_ptr(&self) -> *mut T {
        return self.ptr as *mut T;
    }
}

/// A MPMC Queue based on Dmitry Vyukov's queue.  
/// However, there is a slight modification where head and tail can be locked, as my implementation of Dmitry's queue failed some tests under peak contention  - and I've opted for a more conservative queue
pub struct Queue{
	capacity : usize,
	capacity_mask : u32,
	_cache_pad_0: [u8; 64],
	allocation_size : usize,
	ring_buffer : Unique<AtomicUsize>,
	_cache_pad_1: [u8; 64],
	head : IndexSpinlock<ZST>,
	_cache_pad_2: [u8; 64],
	tail : IndexSpinlock<ZST>,
	_cache_pad_3: [u8; 64],


}

//enqueue
// count as atomic - load check - fail now
// lock tail -- no more adds
	//recheck count to make sure we didn't add 




struct ZST{}
impl Queue{
	// const ENQUEUED : usize = 1;//1<<31;
	// const ENQUEUED_MASK : usize = !1;//Queue::<{CAPACITY}>::ENQUEUED -1;

	// const MASK : usize = {CAPACITY}- 1;
	pub fn new(capacity : usize)->Queue{
		assert!(capacity.is_power_of_two());
		let alloc_size = core::mem::size_of::<AtomicUsize>() * capacity;
		let allocation = mmap::alloc_page_aligned(alloc_size);
		return Queue{
			capacity : capacity,
			capacity_mask: (capacity - 1) as u32,
			head:IndexSpinlock::<ZST>::new(0, ZST{}),
			tail:IndexSpinlock::<ZST>::new(0, ZST{}),
			ring_buffer: Unique::new(allocation.memory as *mut AtomicUsize),
			allocation_size: allocation.size,
			//ring_buffer: UnsafeCell::new(allocation.memory as *mut [usize;{CAPACITY}]),//unsafe{core::mem::zeroed()},//[unsafe{core::mem::zeroed()};{CAPACITY}],
			 _cache_pad_0: [0;64],
			 _cache_pad_1: [0;64],
			 _cache_pad_2: [0;64],
			 _cache_pad_3: [0;64],
		};
		
	}


	pub fn enqueue(&self, value : NonZeroUsize) ->bool{	

		let v = value.get();
		debug_assert_ne!(v, 0);

		let mut tail = self.tail.lock();
		let tail_value = tail.read();
		unsafe{
			let storage = self.ring_buffer.as_ptr().offset(tail_value as isize).as_ref().unwrap().load(Ordering::Relaxed);
			if(storage != 0){return false;}
			self.ring_buffer.as_ptr().offset(tail_value as isize).as_ref().unwrap().store(v, Ordering::Relaxed);
		}
		// ring_buffer[tail_value as usize] = value;
		tail.write(tail_value.wrapping_add(1) & self.capacity_mask);
		return true;
	}

	pub fn dequeue(&self) -> Option<NonZeroUsize>{

		let mut head = self.head.lock();
		let head_value = head.read();
		let mut storage;
		unsafe{
			storage = self.ring_buffer.as_ptr().offset(head_value as isize).as_ref().unwrap().load(Ordering::Relaxed);;
			if(storage == 0){return None;}
			self.ring_buffer.as_ptr().offset(head_value as isize).as_ref().unwrap().store(0, Ordering::Relaxed);

			// ring_buffer[head_value as usize] = 0;
			head.write(head_value.wrapping_add(1) & self.capacity_mask);
			return Some(NonZeroUsize::new_unchecked(storage));
		}
		
	}
}

impl Drop for Queue {
    fn drop(&mut self) {
    	unsafe{
    		mmap::free_page_aligned(self.ring_buffer.as_ptr() as *mut u8, self.allocation_size);
    	}
	}
}

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

#[cfg(test)]
mod test;
