use crate::mem::QueueU32;
use crate::sync::Unique;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;
use crate::mem::nullable::MaybeNull;
use crate::mem::nullable::Nullable;
const INITIALIZED: u32 = 1; //'in use' flag
const UNIQUE_OFFSET: u32 = 2; // unique value

pub struct ResourceData<T> {
    data: MaybeUninit<T>,
    ref_count: AtomicU32,
    unique_id: AtomicU32,
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct ResourceHandle {
    index: u32,
    unique: u32,
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct ResourceRef<'a, T> {
    index: u32,
    // unique : u32,
    _phantom: PhantomData<*mut u8>, //to disable send and sync
    _lifetime: PhantomData<&'a T>,
}
const REF_NULL: u32 = 0xFFFFFFFF;
impl<'a, T> MaybeNull for ResourceRef<'a, T>{
    fn is_null(&self)->bool{
    	return self.index == REF_NULL;
    }
     fn null()->ResourceRef<'a, T>{
    	return  ResourceRef {
       	 	index: REF_NULL, 
        	_phantom: PhantomData,
            _lifetime: PhantomData,
        };
    }
    /// Takes the value out , leaving a null in its place.
    fn take(&mut self)->ResourceRef<'a, T>{
    	return  ResourceRef {
       	 	index: core::mem::replace(&mut self.index, REF_NULL), 
        	_phantom: PhantomData,
            _lifetime: PhantomData,
        };
    }
    fn replace(&mut self, new : ResourceRef<'a, T>)->ResourceRef<'a, T>{
    	return  ResourceRef {
       	 	index: core::mem::replace(&mut self.index, new.index), 
        	_phantom: PhantomData,
            _lifetime: PhantomData,
        };
    }
}
pub struct ResourceManager<'a, T> {
    buffer: Unique<ResourceData<T>>,

    free_queue: QueueU32<'a>,
    high_water_mark: AtomicU32,
    capacity: u32,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> ResourceManager<'a, T> {
    pub const unsafe fn from_static(
        free_queue: &'a *mut AtomicU32,
        buffer: &'a *mut ResourceData<T>,
        capacity: u32,
    ) -> ResourceManager<'a, T> {
        return ResourceManager::<'a, T> {
            buffer: Unique::<ResourceData<T>>::new(*buffer),
            free_queue: QueueU32::from_static(free_queue, capacity as usize),
            capacity: capacity,
            high_water_mark: AtomicU32::new(0),
            _lifetime: PhantomData,
        };
    }

    unsafe fn get_data(&self, index: u32) -> &mut ResourceData<T> {
        return self
            .buffer
            .as_ptr()
            .offset(index as isize)
            .as_mut()
            .unwrap();
    }

    unsafe fn increment_ref_count(&self, index: u32) {
        let data = self.get_data(index);
        data.ref_count.fetch_add(1, Ordering::Release);
    }

    unsafe fn decrement_ref_count(&self, index: u32) {
        let data = self.get_data(index);
        let previous = data.ref_count.fetch_sub(1, Ordering::AcqRel);

        // There is no possible way to get another reference if this was the last one.  We MUST have already incremented the unique value.
        if previous == 1 {
            // Sledgehammer this into mutable - we've protected it behind an atomic ref count.
            ptr::drop_in_place(data.data.as_mut_ptr());

            self.free_queue.enqueue(index as u32);
        }
    }

    /// Store a T in the resource manager.  If space exists, this returns a handle to the object.  Otherwise returns None.
    pub fn store(&self, obj: T) -> Option<ResourceHandle> {
        let tmp = self.free_queue.dequeue();
        let index;
        let unique;
        let data;
        if tmp.is_none() {
            // Assume this operation is going to succeed by incrementing the counter.
            // If it fails, we've run out of memory and things are probably about to get a lot worse, anyway.
            let next = self.high_water_mark.fetch_add(1, Ordering::Relaxed);
            if next >= self.capacity {
                // Ensure the counter does not overflow by continually incrementing.
                self.high_water_mark.store(self.capacity, Ordering::Relaxed);
                return None;
            }

            index = next;
            unique = INITIALIZED;
            data = unsafe { self.get_data(index) };
        } else {
            index = tmp.unwrap();
            data = unsafe { self.get_data(index) };
            unique = data.unique_id.load(Ordering::Acquire) | INITIALIZED;
        }
        data.unique_id.store(unique, Ordering::Release);
        data.data = MaybeUninit::new(obj);

        data.ref_count.store(1, Ordering::Release);

        return Some(ResourceHandle {
            index: index,
            unique: unique,
        });
    }

    /// Release the local reference to the object stored at the handle location.  
    /// The object will not actually be dropped until all references are released, however no handles will return the object.
    pub fn free(&self, handle: ResourceHandle) -> bool {
        unsafe {
            let next = (handle.unique & !INITIALIZED).wrapping_add(UNIQUE_OFFSET);
            let data = self.get_data(handle.index);
            match data.unique_id.compare_exchange(
                handle.unique,
                next,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.decrement_ref_count(handle.index);
                    return true;
                }

                Err(_) => {
                    return false;
                }
            }
        }
    }
}

impl<'a, T: Sync> ResourceManager<'a, T> {
    /// Release a reference previously allocated from the resource manager.
    pub fn release(&self, resource: ResourceRef<'a, T>) {
        unsafe {
            self.decrement_ref_count(resource.index);
        }
    }

    /// Clones a reference, incrementing the reference count.
    pub fn clone(&'a self, resource: &ResourceRef<'a, T>) -> ResourceRef<'a, T> {
        unsafe {
            self.increment_ref_count(resource.index);

            return ResourceRef {
                index: resource.index,
                _phantom: PhantomData,
                _lifetime: PhantomData,
            };
        }
    }
    pub fn get(&'a self, resource: &'a ResourceRef<'a, T>) -> &'a T {
        unsafe {
            let data = self.get_data(resource.index);
            return data.data.as_ptr().as_ref().unwrap();
        }
    }
    /// Get a reference counted reference to the object based on a handle.  Returns None if the handle points to empty space.
    pub fn retain(&'a self, handle: ResourceHandle) -> Nullable<ResourceRef<'a, T>> {
        unsafe {
            self.increment_ref_count(handle.index);
            let data = self.get_data(handle.index);
            if data.unique_id.load(Ordering::Acquire) != handle.unique {
                self.decrement_ref_count(handle.index);
                // println!("none {} {}", index, unique);
                return Nullable::null();
            } else {
                return Nullable::new(ResourceRef {
                    index: handle.index,
                    _phantom: PhantomData,
                    _lifetime: PhantomData,
                });
            }
        }
    }
}

#[cfg(test)]
mod test;
