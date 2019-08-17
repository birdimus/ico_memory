use core::sync::atomic::AtomicUsize;
use core::num::NonZeroUsize;
use core::sync::atomic::Ordering;
use crate::sync::index_lock::IndexSpinlock;
const CAPACITY : usize = 2097152;

/// A MPMC Queue based on Dmitry Vyukov's queue.  
/// However, there is a slight modification where head and tail can be locked, as my implementation of Dmitry's queue failed some tests under peak contention  - and I've opted for a more conservative queue

#[repr(C)]
pub struct Queue{
	_cache_pad_0: [u8; 64],
	buffer : [usize; CAPACITY],
	_cache_pad_1: [u8; 64],
	head : IndexSpinlock,
	_cache_pad_2: [u8; 64],
	tail : IndexSpinlock,
	_cache_pad_3: [u8; 64],

}


impl Queue{
	const CAPACITY_MASK : u32 = CAPACITY as u32 - 1;

	pub const fn new()->Queue{
		return Queue{

			head:IndexSpinlock::new(0),
			tail:IndexSpinlock::new(0),
			buffer : [0; CAPACITY],
			 _cache_pad_0: [0;64],
			 _cache_pad_1: [0;64],
			 _cache_pad_2: [0;64],
			 _cache_pad_3: [0;64],
		};
		
	}

	#[inline(always)]
	fn get_storage(&self, index : usize)->&AtomicUsize{
		unsafe{
			return &*(&self.buffer[index] as *const usize as *const AtomicUsize);
		}
	}

	pub fn enqueue(&self, value : NonZeroUsize) ->bool{	

		let v = value.get();
		debug_assert_ne!(v, 0);

		let mut tail = self.tail.lock();
		let tail_value = tail.read();

		let storage = self.get_storage(tail_value as usize);
		let stored_value = storage.load(Ordering::Relaxed);
		if(stored_value != 0){return false;}
		storage.store(v, Ordering::Relaxed);
		tail.write(tail_value.wrapping_add(1) & Queue::CAPACITY_MASK);
		return true;
	}

	pub fn dequeue(&self) -> Option<NonZeroUsize>{

		let mut head = self.head.lock();
		let head_value = head.read();
		let storage = self.get_storage(head_value as usize);
		let stored_value = storage.load(Ordering::Relaxed);
		if(stored_value == 0){return None;}
		storage.store(0, Ordering::Relaxed);
		head.write(head_value.wrapping_add(1) & Queue::CAPACITY_MASK);
		unsafe{
			return Some(NonZeroUsize::new_unchecked(stored_value));
		}
		
	}
}


unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

#[cfg(test)]
mod test;
