use crate::sync::rw_lock::RWSpinLock;
use crate::sync::index_lock::IndexSpinlock;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::mem::queue::Queue;
use crate::mem::queue::Swap;
use core::num::NonZeroUsize;
use core::marker::PhantomData;
use core::mem::MaybeUninit;

const DESTROY_LOCK: u64 = 1 << 63;
const DESTROY_REQUEST: u64 = 1 << 62;
const INDEX_MASK: u64 = 0x00000000FFFFFFFF;

const DESTROY_LOCK_32: u32 = 1 << 31;
const DESTROY_REQUEST_32: u32 = 1 << 30;

const DESTROY_MASK_32: u32 = !(DESTROY_LOCK_32 | DESTROY_REQUEST_32);

#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct Handle{
	index : u32,
	unique : u32,
}


/// A weak shared reference.
// struct Reference<'a>{
// 	manager : &'a ResourceManager<'a>,
// 	handle : Handle,
// }

struct Resource<'a, T>{
	manager : &'a ResourceManager<'a, T>,
	handle : Handle,
	_phantom : PhantomData<T>,
}
unsafe impl<'a, T: Send> Send for Resource<'a, T> {}
unsafe impl<'a, T: Sync + Send> Sync for Resource<'a, T> {}
impl<'a, T> Resource<'a, T> {
	pub fn destroy(&self){
		// self.manager.ref_count[self.index as usize].fetch_or(DESTROY_REQUEST, Ordering::Release);
	}
}
impl<'a, T> Drop for Resource<'a, T> {
    fn drop(&mut self) {
    	self.manager.decrement_resource_counter(self.handle.index);
    }
}
impl<'a, T> Clone for Resource<'a, T> {
    fn clone(&self) -> Self {
    	return self.manager.clone_resource(self);
    }
}




/// A thread safe resource manager.
/// This is similar to atomic shared pointers (arc), and weak pointers, with a few differences
/// First, resources are stored in fixed size pools - and the handles point to those slots.  
/// Slots are reused with a 30 bit unique counter.
/// Second, resources can be marked as destroyed - blocking all future accesses - but the resources are not actually destroyed until the final strong reference is dropped.
struct ResourceManager<'a, T>{
	data : &'a [MaybeUninit<T>],
	ref_count : &'a [AtomicU64],
	free_queue : Queue<'a>,
	capacity: usize,
	initialized: AtomicBool,
	_phantom : PhantomData<T>,
}

impl<'a, T> ResourceManager<'a, T> {

	pub const fn new(
		data: &'a [MaybeUninit<T>],
        slots: &'a [AtomicU64],
        queue: &'a [AtomicUsize],
        capacity: usize,
    ) -> ResourceManager<'a, T> {

        return ResourceManager {
        	data:data,
        	capacity : capacity,
            ref_count : slots,
            free_queue: Queue::new(queue, capacity),
            initialized: AtomicBool::new(false),
            _phantom : PhantomData,
        };
    }
    fn init(&self){
    	let init = self.initialized.swap(true, Ordering::Acquire);
    	if(!init){
    		for i in 0..self.capacity{
    			let index_plus_one = unsafe{NonZeroUsize::new_unchecked(i as usize + 1)};
    			self.free_queue.enqueue(index_plus_one);
    		}
    	}
    }
    fn clone_resource(&'a self, resource : &Resource<'a, T>)->Resource<'a, T>{
		let ref_count = &self.ref_count[resource.handle.index as usize];
		let counter = ref_count.fetch_add(1, Ordering::Acquire);
		let unique = ((counter>>32) as u32)&!DESTROY_REQUEST_32;
		std::debug_assert_eq!(unique, resource.handle.unique);

		
		return Resource{
			manager : self,
			handle : resource.handle,
			_phantom : PhantomData,
		};

	}

	fn create_resource(&'a self) -> Option<Handle>{
		let index_plus_one = self.free_queue.dequeue();
		if(index_plus_one.is_none()){return None;}
		let index = index_plus_one.unwrap().get() as u32-1;
		let ref_count = &self.ref_count[ (index) as usize];
		let counter = ref_count.load(Ordering::Acquire);
		let unique = ((counter>>32) as u32);
		return Some(Handle{
			index:index, 
			unique:unique,
		});

	}

	fn get_resource(&'a self, handle : Handle)->Option<Resource<'a, T>>{
		let ref_count = &self.ref_count[handle.index as usize];
		let counter = ref_count.fetch_add(1, Ordering::Acquire);
		let unique = (counter>>32) as u32;
		if(unique != handle.unique){
			self.decrement_resource_counter(handle.index);
			return None;
		}
		
		return Some(Resource{
			manager : self,
			handle : handle,
			_phantom : PhantomData,
		});

	}

	fn decrement_resource_counter(&self, index : u32){
		let ref_count = &self.ref_count[index as usize];
		let counter = ref_count.fetch_sub(1, Ordering::Acquire);
		let unique = (counter>>32) as u32;
		let references = INDEX_MASK & (counter-1);

		if(unique & DESTROY_REQUEST_32 == DESTROY_REQUEST_32 && references == 0){
			let current = counter-1;

			// This masks all bits of the unique value off and adds a destroy lock
			let target = (references) | DESTROY_LOCK;
			match ref_count.compare_exchange(current, target, Ordering::SeqCst, Ordering::Relaxed){
				Ok(_) => {
					//TODO: We could check for wraparound and abort here.
					let new_unique_xor : u64 = ((((unique + 1)&!DESTROY_REQUEST_32) | DESTROY_LOCK_32 ) as u64) <<32;

					//We should have a new unique id that no one knows about.  
					ref_count.fetch_xor(new_unique_xor, Ordering::Release);

					 println!("new unique {}", new_unique_xor);
					 let index_plus_one = unsafe{NonZeroUsize::new_unchecked(references as usize + 1)};
					 self.free_queue.enqueue(index_plus_one);
                    // Increment the unique a
                }
                Err(x) => {},
			}
		}
	}

	fn destroy(&self, handle : Handle){
		let ref_count = &self.ref_count[handle.index as usize];
		let mut value = ref_count.load(Ordering::Acquire);
		let unique = (value>>32) as u32;
		let references = INDEX_MASK & (value);

		// This will fail if there is a pending destroy request or lock, or the unique value has changed.
		while unique == handle.unique{
			let target = value | DESTROY_REQUEST;
			match ref_count.compare_exchange_weak(value, target, Ordering::Release, Ordering::Acquire){
				Ok(_) => {return;},
				Err(x) => {
					value = x;
					core::sync::atomic::spin_loop_hint();
				},
			}

		}

	}
}






