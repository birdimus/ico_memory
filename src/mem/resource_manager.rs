use crate::mem::queue::Queue;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;


use crate::mem::queue::Queue32;

const INITIALIZED : u32 = 1; //'in use' flag
const UNIQUE_OFFSET : u32 = 2; // unique value

//32 bit index + 1 bit 'in-use' flag + 32 bit unique
const INDEX_MASK : u64 = (1<<32)-1;



pub struct ResourceManager<'a, T : Sync>{
	reference_counts: &'a [AtomicU32],
	slots: &'a [AtomicU32],
	data: &'a [MaybeUninit<T>],
	free_queue: Queue32<'a>,
	high_water_mark : AtomicU32,
	capacity : u32,
}


impl<'a, T: Sync> ResourceManager<'a, T>{

	pub const fn new(slots: &'a [AtomicU32], 
		reference_counts: &'a [AtomicU32], 
		free_queue: &'a [AtomicU32], 
		data: &'a [MaybeUninit<T>],
		capacity: u32) ->ResourceManager<'a, T>{
		return ResourceManager::<'a, T>{
			reference_counts: reference_counts,
			slots:slots,
			data:data,
			free_queue : Queue32::new(free_queue, capacity as usize),
			capacity : capacity,
			high_water_mark : AtomicU32::new(0),
		};
	}

	fn increment_ref_count(&self, index : usize){
		self.reference_counts[index].fetch_add(1, Ordering::Release);
	}


	fn decrement_ref_count(&self, index : usize){
		let ref_count = &self.reference_counts[index];
		let previous = ref_count.fetch_sub(1, Ordering::AcqRel);

		// There is no possible way to get another reference if this was the last one.  We must have already incremented the index.
		if previous == 1{

			unsafe{
				// Sledgehammer this into mutable - we've protected it behind an atomic ref count.
				core::ptr::drop_in_place(self.data[index].as_ptr() as * mut T);

				self.free_queue.enqueue(index as u32);
			}
			
		}
	}

	pub unsafe fn release_reference(&self, reference : &'a T){
		let ptr = reference as *const T;
		let index = ((ptr as isize - self.data.as_ptr() as isize)/core::mem::size_of::<T>() as isize);
		if index < 0 || index >= self.capacity as isize {
			// panic!("Releasing an object we don't own!!!!");
		}

		self.decrement_ref_count(index as usize);
	}

	/// Clones a reference, incrementing the reference count.
	pub unsafe fn clone_reference(&self, reference : &'a T) ->&'a T{
		let ptr = reference as *const T;
		let index = ((ptr as isize - self.data.as_ptr() as isize)/core::mem::size_of::<T>() as isize) ;
		if index < 0 || index >= self.capacity as isize {
			// panic!("cloning an object we don't own!!!!");
		}
		//Since we already have one, it's safe to get another.
		self.increment_ref_count(index as usize);

		return reference;
	}

	/// Get a reference counted reference to the object based on a handle.  Returns None if the handle points to empty space.
	pub unsafe fn retain_reference(&self, handle : u64) ->Option<&'a T>{
		let index = (handle & INDEX_MASK) as usize;
		let unique = (handle >> 32) as u32;
		self.increment_ref_count(index);
		

		if self.slots[index as usize].load(Ordering::Acquire) != unique{
			self.decrement_ref_count(index);
			// println!("none {} {}", index, unique);
			return None;
		}
		else{
			unsafe{
				
			return self.data[index as usize].as_ptr().as_ref().map(|val| val);
			}
		}
	}

	/// Store a T in the resource manager.  If space exists, this returns a handle to the object.  Otherwise returns None.
	pub fn retain(&self, obj : T) -> Option<u64>{
		let tmp = self.free_queue.dequeue();
		let mut index;
		let mut unique;
		if(tmp.is_none()){
			
			// Assume this operation is going to succeed by incrementing the counter.
			// If it fails, we've run out of memory and things are probably about to get a lot worse, anyway.
			let next = self.high_water_mark.fetch_add(1, Ordering::Relaxed);
			if(next >= self.capacity){
				// Ensure the counter does not overflow by continually incrementing.
				self.high_water_mark.store(self.capacity, Ordering::Relaxed);
				return None;
			}

			index = next as usize;
			unique = INITIALIZED;
			
		}
		else{
			index = tmp.unwrap() as usize;
			unique = self.slots[index].load(Ordering::Acquire) | INITIALIZED;

		}
		self.slots[index].store(unique, Ordering::Release);
		let mut t = self.data[index as usize].as_ptr() as * mut T;
		unsafe{core::ptr::write(t, obj)};
		self.reference_counts[index as usize].store(1, Ordering::Release);

		return Some( (index as u64) | ((unique as u64) <<32));
	}

	/// Release the local reference to the object stored at the handle location.  
	/// The object will not actually be dropped until all references are released, however no handles will return the object.
	pub fn release(&self, handle : u64)->bool {
		let index = (handle & INDEX_MASK) as usize;
		let unique = (handle >> 32) as u32;
		let next = (unique & !INITIALIZED).wrapping_add(UNIQUE_OFFSET);

		match self.slots[index].compare_exchange(unique,
			next,Ordering::AcqRel, Ordering::Relaxed){
			Ok(_)=>{
				self.decrement_ref_count(index);
				return true;
			},

			Err(x)=>{return false;},
		}
	}
}





#[cfg(test)]
mod test;
