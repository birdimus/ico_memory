use core::mem::MaybeUninit;
use core::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use crate::mem::queue::Queue32;
use core::marker::PhantomData;

const INITIALIZED: u32 = 1; //'in use' flag
const UNIQUE_OFFSET: u32 = 2; // unique value

//32 bit index + 1 bit 'in-use' flag + 32 bit unique
const INDEX_MASK: u64 = (1 << 32) - 1;


struct Unique<T> {
    ptr: *const T,              // *const for variance
    _marker: PhantomData<T>,    // For the drop checker
}

// Deriving Send and Sync is safe because we are the Unique owners
// of this data. It's like Unique<T> is "just" T.
unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    pub const fn new(ptr: *mut T) -> Self {
        Unique { ptr: ptr, _marker: PhantomData }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr as *mut T
    }
}

pub struct ResourceManager<T> {
    // reference_counts: &'a [AtomicU32],
    reference_counts: Unique<AtomicU32>,
    // slots: &'a [AtomicU32],
    slots: Unique<AtomicU32>,
    // data: &'a [MaybeUninit<T>],
    data: Unique<T>,
    free_queue: Queue32,
    high_water_mark: AtomicU32,
    capacity: u32,
}

impl<T> ResourceManager<T> {
    pub const fn new(
        slots: *mut AtomicU32,
        reference_counts: *mut AtomicU32,
        free_queue: *mut AtomicU32,
        // data: &'a [MaybeUninit<T>],
        data: *mut T,
        capacity: u32,
    ) -> ResourceManager<T> {

        return ResourceManager::<T> {
            reference_counts: Unique::new(reference_counts),
            slots: Unique::new(slots),
            data: Unique::<T>::new(data),
            free_queue: Queue32::new(free_queue, capacity as usize),
            capacity: capacity,
            high_water_mark: AtomicU32::new(0),
        };
    }

    fn increment_ref_count(&self, index: isize) {
    	let ref_count = unsafe{self.reference_counts.as_ptr().offset(index).as_ref().unwrap()};
        ref_count.fetch_add(1, Ordering::Release);
    }

    fn decrement_ref_count(&self, index: isize) {
        let ref_count = unsafe{self.reference_counts.as_ptr().offset(index).as_ref().unwrap()};
        let previous = ref_count.fetch_sub(1, Ordering::AcqRel);

        // There is no possible way to get another reference if this was the last one.  We must have already incremented the index.
        if previous == 1 {
            unsafe {
                // Sledgehammer this into mutable - we've protected it behind an atomic ref count.
                core::ptr::drop_in_place(self.data.as_ptr().offset(index) as *mut T);

                self.free_queue.enqueue(index as u32);
            }
        }
    }

    /// Store a T in the resource manager.  If space exists, this returns a handle to the object.  Otherwise returns None.
    pub fn retain(&self, obj: T) -> Option<u64> {
        let tmp = self.free_queue.dequeue();
        let mut index;
        let mut unique;
        
        if tmp.is_none() {
            // Assume this operation is going to succeed by incrementing the counter.
            // If it fails, we've run out of memory and things are probably about to get a lot worse, anyway.
            let next = self.high_water_mark.fetch_add(1, Ordering::Relaxed);
            if next >= self.capacity {
                // Ensure the counter does not overflow by continually incrementing.
                self.high_water_mark.store(self.capacity, Ordering::Relaxed);
                return None;
            }

            index = next as isize;
            unique = INITIALIZED;

        } else {
            index = tmp.unwrap() as isize;
            let slot = unsafe{self.slots.as_ptr().offset(index).as_ref().unwrap()};
            unique = slot.load(Ordering::Acquire) | INITIALIZED;
        }
        let slot = unsafe{self.slots.as_ptr().offset(index).as_ref().unwrap()};
        slot.store(unique, Ordering::Release);
        unsafe{
        	let mut t = self.data.as_ptr().offset(index as isize) as *mut T;
         	core::ptr::write(t, obj) ;
     	}
     	let ref_count = unsafe{self.reference_counts.as_ptr().offset(index).as_ref().unwrap()};
        ref_count.store(1, Ordering::Release);

        return Some((index as u64) | ((unique as u64) << 32));
    }

    /// Release the local reference to the object stored at the handle location.  
    /// The object will not actually be dropped until all references are released, however no handles will return the object.
    pub fn release(&self, handle: u64) -> bool {
        let index = (handle & INDEX_MASK) as isize;
        let unique = (handle >> 32) as u32;
        let next = (unique & !INITIALIZED).wrapping_add(UNIQUE_OFFSET);
        let slot = unsafe{self.slots.as_ptr().offset(index).as_ref().unwrap()};
        match slot.compare_exchange(unique, next, Ordering::AcqRel, Ordering::Relaxed)
        {
            Ok(_) => {
                self.decrement_ref_count(index);
                return true;
            }

            Err(x) => {
                return false;
            }
        }
    }
}

impl<T: Sync> ResourceManager<T> {
    pub unsafe fn release_reference(&self, reference: & T) {
        let ptr = reference as *const T;
        let index =
            (ptr as isize - self.data.as_ptr() as isize) / core::mem::size_of::<T>() as isize;
        // if index < 0 || index >= self.capacity as isize {
        // panic!("Releasing an object we don't own!!!!");
        // }

        self.decrement_ref_count(index);
    }

    /// Clones a reference, incrementing the reference count.
    pub unsafe fn clone_reference<'a>(&'a self, reference: &'a T) -> &'a T {
        let ptr = reference as *const T;
        let index =
            (ptr as isize - self.data.as_ptr() as isize) / core::mem::size_of::<T>() as isize;
        // if index < 0 || index >= self.capacity as isize {
        // panic!("cloning an object we don't own!!!!");
        // }
        //Since we already have one, it's safe to get another.
        self.increment_ref_count(index);

        return reference;
    }

    /// Get a reference counted reference to the object based on a handle.  Returns None if the handle points to empty space.
    pub unsafe fn retain_reference(& self, handle: u64) -> Option<& T> {
        let index = (handle & INDEX_MASK) as isize;
        let unique = (handle >> 32) as u32;
        self.increment_ref_count(index);
        let slot = self.slots.as_ptr().offset(index).as_ref().unwrap();
        if slot.load(Ordering::Acquire) != unique {
            self.decrement_ref_count(index);
            // println!("none {} {}", index, unique);
            return None;
        } else {
            unsafe {
                return self.data.as_ptr().offset(index).as_ref().map(|val| val);
            }
        }
    }
}

#[cfg(test)]
mod test;
