use core::cell::UnsafeCell;
use core::sync::atomic::AtomicU32;
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


pub struct Queue<T, const CAPACITY: usize>{
	_cache_pad_0: [u8; 64],
	//ring_buffer : UnsafeCell< *mut [usize;{CAPACITY}]>,
	allocation_size : usize,
	ring_buffer : Unique<Option<T>>,
	_cache_pad_1: [u8; 64],
	head : IndexSpinlock<ZST>,
	_cache_pad_2: [u8; 64],
	tail : IndexSpinlock<ZST>,
	_cache_pad_3: [u8; 64],
	_marker: PhantomData<T>, 

}

//enqueue
// count as atomic - load check - fail now
// lock tail -- no more adds
	//recheck count to make sure we didn't add 

struct ZST{}

impl <T, const CAPACITY: usize> Queue<T, {CAPACITY}>{
	const ENQUEUED : usize = 1;//1<<31;
	const ENQUEUED_MASK : usize = !1;//Queue::<{CAPACITY}>::ENQUEUED -1;

	// const MASK : usize = {CAPACITY}- 1;
	pub fn new()->Queue<T, {CAPACITY}>{
		let alloc_size = core::mem::size_of::<Option<T>>() * CAPACITY;
		let allocation = mmap::alloc_page_aligned(alloc_size);
		return Queue::<T, {CAPACITY}>{
			head:IndexSpinlock::<ZST>::new(0, ZST{}),
			tail:IndexSpinlock::<ZST>::new(0, ZST{}),
			ring_buffer: Unique::new(allocation.memory as *mut Option<T>),
			allocation_size: allocation.size,
			//ring_buffer: UnsafeCell::new(allocation.memory as *mut [usize;{CAPACITY}]),//unsafe{core::mem::zeroed()},//[unsafe{core::mem::zeroed()};{CAPACITY}],
			 _cache_pad_0: [0;64],
			 _cache_pad_1: [0;64],
			 _cache_pad_2: [0;64],
			 _cache_pad_3: [0;64],
			 _marker: PhantomData,
		};
		
	}


	pub fn enqueue(&self, value : T) ->bool{	

		let mut tail = self.tail.lock();
		let tail_value = tail.read();
		unsafe{
		// let mut ring_buffer = unsafe { &mut *self.ring_buffer.get() };
		let storage = ptr::read(self.ring_buffer.as_ptr().offset(tail_value as isize));// ring_buffer[tail_value as usize];
		if(storage.is_some()){return false;}
			ptr::write(self.ring_buffer.as_ptr().offset(tail_value as isize), Some(value));
		}
		// ring_buffer[tail_value as usize] = value;
		tail.write(tail_value.wrapping_add(1) & (CAPACITY as u32-1));
		return true;
	}

	pub fn dequeue(&self) -> Option<T>{

		let mut head = self.head.lock();
		let head_value = head.read();
		let mut storage;
		unsafe{
			// let mut ring_buffer = unsafe { &mut *self.ring_buffer.get() };
			storage = ptr::read(self.ring_buffer.as_ptr().offset(head_value as isize));
			// let storage = ring_buffer[head_value as usize];
			if(storage.is_none()){return None;}
			
			ptr::write(self.ring_buffer.as_ptr().offset(head_value as isize), None);
		}
		// ring_buffer[head_value as usize] = 0;
		head.write(head_value.wrapping_add(1) & (CAPACITY as u32-1));
		return storage;
	}
}

impl<T, const CAPACITY: usize> Drop for Queue<T, {CAPACITY}> {
    fn drop(&mut self) {
    	unsafe{
    	mmap::free_page_aligned(self.ring_buffer.as_ptr() as *mut u8, self.allocation_size);
    	}
	}
}

unsafe impl<T : Send, const CAPACITY: usize> Send for Queue<T, {CAPACITY}> {}
unsafe impl<T : Sync, const CAPACITY: usize> Sync for Queue<T, {CAPACITY}> {}

