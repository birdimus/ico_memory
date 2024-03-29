use core::cell::Cell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;
use core::ptr;
pub struct IndexedData<T> {
    data: MaybeUninit<T>,
    unique: Cell<u32>,
    ref_count: Cell<u32>,
}
impl<T> IndexedData<T> {
    fn init(&mut self, value: T) {
        self.data = MaybeUninit::new(value);
        self.ref_count.set(0);
        self.unique.set(1);
    }
    fn reinit(&mut self, value: T) {
        self.data = MaybeUninit::new(value);
        self.ref_count.set(0);
        self.set_initialized();
    }
    fn is_initialized(&self) -> bool {
        return (self.unique.get() & 1) == 1;
    }
    fn set_destroyed(&mut self) {
        self.unique.set(self.unique.get() | 2);
    }
    fn is_destroyed(&mut self) -> bool {
        return (self.unique.get() & 2) == 2;
    }

    fn set_initialized(&mut self) {
        self.unique.set(self.unique.get() | 1);
    }
    fn increment_unique(&mut self) {
        self.unique.set(self.unique.get() + 4);
    }
    //clear init and destroyed
    fn set_uninitialized(&mut self) {
        self.unique.set(self.unique.get() & !3);
    }
    fn is_match(&self, unique: u32) -> bool {
        return self.unique.get() == unique;
    }
}

const SLOT_NULL: u32 = 0xFFFFFFFF;
const NON_NULL_BIT: u32 = 1 << 31;
const REF_MASK: u32 = !NON_NULL_BIT;

pub struct IndexedDataStore<'a, T> {
    buffer: *mut MaybeUninit<IndexedData<T>>, //definitely not ever thread safe.
    capacity: u32,
    high_water_mark: Cell<u32>,
    free_stack: Cell<u32>,
    active_count: Cell<u32>,
    destroyed_count: Cell<u32>,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> IndexedDataStore<'a, T> {
    pub const unsafe fn from_raw(
        data: &'a *mut MaybeUninit<IndexedData<T>>,
        capacity: u32,
    ) -> IndexedDataStore<'a, T> {
        return IndexedDataStore {
            buffer: *data,
            capacity: capacity,
            high_water_mark: Cell::new(0),
            free_stack: Cell::new(SLOT_NULL),
            active_count: Cell::new(0),
            destroyed_count: Cell::new(0),
            _lifetime: PhantomData,
        };
    }
    // unsafe fn get_raw(&'a self, index:u32) ->&'a T{
    // 	return self.buffer.offset(index as isize).as_ref().unwrap().data.as_ptr().as_ref().unwrap();
    // }
    #[inline(always)]
    unsafe fn get_data(&self, index: u32) -> &mut IndexedData<T> {
        return self
            .buffer
            .offset(index as isize)
            .as_mut()
            .unwrap()
            .as_mut_ptr()
            .as_mut()
            .unwrap();
    }
    unsafe fn decrement_ref_count(&self, index: u32) {
        let data = self.get_data(index);
        data.ref_count.set(data.ref_count.get() - 1);
        // println!("decrement {}", data.ref_count.get());
        if data.ref_count.get() == 0 && data.is_destroyed() {
        	// println!("actual drop");
            data.set_uninitialized();
            data.increment_unique();
            ptr::drop_in_place(data.data.as_mut_ptr());
            data.ref_count.set(self.free_stack.get());
            self.free_stack.set(index);
            self.destroyed_count.set(self.destroyed_count.get() - 1);
        }
    }

    /// How much of the buffer has ever been used.
    #[inline(always)]
    pub fn high_water_mark(&self) -> u32 {
        return self.high_water_mark.get();
    }

    /// How many objects are currently stored and accessible.
    #[inline(always)]
    pub fn active_count(&self) -> u32 {
        return self.active_count.get();
    }

    /// How many objects are currently stored but marked destroyed.
    #[inline(always)]
    pub fn destroyed_count(&self) -> u32 {
        return self.destroyed_count.get();
    }

    /// How much free space.  Capcity - (active + destroyed).
    #[inline(always)]
    pub fn available_count(&self) -> u32 {
        return self.capacity() - self.active_count() - self.destroyed_count();
    }

    /// Total fixed capacity.
    #[inline(always)]
    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }

    /// Retain a strong reference.  If no reference exists returns None, otherwise, this increments the reference count and returns the reference.  
    /// It is up to the user to manually release() the returned reference when they are finished.
    pub fn retain(&'a self, handle: IndexedHandle) -> Option<IndexedRef<'a, T>> {
        if handle.index >= self.high_water_mark.get() {
            return None;
        }
        unsafe {
            let data = self.get_data(handle.index);
            if !data.is_match(handle.unique.get()) {
                return None;
            }
            data.ref_count.set(data.ref_count.get() + 1);
            return Some(IndexedRef {
                index: NonZeroU32::new_unchecked(handle.index | NON_NULL_BIT),
                _phantom: PhantomData,
                _lifetime: PhantomData,
            });
            //return data.data.as_ptr().as_ref();
        }
    }

    /// It is imperative users not hold these references
    #[inline(always)]
    pub unsafe fn get(&'a self, reference: &'a IndexedRef<T>) -> &'a mut T {
        // unsafe {
        let data = self.get_data(reference.index.get() & REF_MASK);
        return data.data.as_mut_ptr().as_mut().unwrap();
        // return self.get_raw(reference.index);
        // }
    }
    #[inline(always)]
    pub fn handle(&'a self, reference: &IndexedRef<'a, T>) -> IndexedHandle {
    	unsafe {
    		let idx = reference.index.get() & REF_MASK;
            let data = self.get_data(idx);

            return IndexedHandle {
                    index: idx,
                    unique: NonZeroU32::new_unchecked(data.unique.get()) ,
                    _phantom: PhantomData,
                };
        }
    }
    //    pub fn invoke<F>(&'a self, reference: &'a IndexedRef<T>, closure: F)->u32
    //     where F: Fn(&mut T)->u32 {

    //     unsafe {
    //            let data = self.get_data(reference.index);
    //            let t_value = data.data.as_mut_ptr().as_mut().unwrap();
    //            return closure(t_value);
    //            // return self.get_raw(reference.index);
    //        }

    // }

    //since we already have a reference, it's safe to get another
    pub fn clone(&'a self, reference: &IndexedRef<'a, T>) -> IndexedRef<'a, T> {
        unsafe {
            let data = self.get_data(reference.index.get() & REF_MASK);
            data.ref_count.set(data.ref_count.get() + 1);
            return IndexedRef {
                index: reference.index,
                _phantom: PhantomData,
                _lifetime: PhantomData,
            };
            // return self.get_raw(reference.index);
        }
    }
    pub fn destroyed(&'a self, reference: &IndexedRef<'a, T>) -> bool {
        unsafe {
            let data = self.get_data(reference.index.get() & REF_MASK);
            return data.is_destroyed();
        }
    }
    /// Release a strong reference.  This decrements the reference count.
    pub fn release(&'a self, reference: IndexedRef<T>) {
        unsafe {
            self.decrement_ref_count(reference.index.get() & REF_MASK);
            // return Handle{index:reference.index, unique:reference.unique, _phantom:PhantomData};
        }
    }

    pub fn store(&self, value: T) -> Option<IndexedHandle> {
        let free = self.free_stack.get();
        if free != SLOT_NULL {
            let data = unsafe { self.get_data(free) };
            self.free_stack.set(data.ref_count.get());

            data.reinit(value);

            let active_count = self.active_count.get();
            self.active_count.set(active_count + 1);
            return Some(IndexedHandle {
                index: free,
                unique: unsafe { NonZeroU32::new_unchecked(data.unique.get()) },
                _phantom: PhantomData,
            });
        } else {
            let hwm = self.high_water_mark.get();
            if hwm < self.capacity {
                self.high_water_mark.set(hwm + 1);
                // println!("index {} {}", result_index, self.buffer as usize);

                let data = unsafe { self.get_data(hwm) };
                data.init(value);

                let active_count = self.active_count.get();
                self.active_count.set(active_count + 1);
                return Some(IndexedHandle {
                    index: hwm,
                    unique: unsafe { NonZeroU32::new_unchecked(data.unique.get()) },
                    _phantom: PhantomData,
                });
            } else {
                // panic!("Out of data storage.");
                // #[cfg(any(test, feature = "std"))]
                // std::process::abort();
                return None;
            }
        }
    }
    /// Release a strong reference.  This decrements the reference count.
    pub fn free(&'a self, reference: IndexedRef<T>) {
        unsafe {
            let idx = reference.index.get() & REF_MASK;
            let data = self.get_data(idx);

            if !data.is_destroyed() {
                //This marks the element 'destroyed'. Safe to call multiple times
                data.set_destroyed();
                self.active_count.set(self.active_count.get() - 1);
                self.destroyed_count.set(self.destroyed_count.get() + 1);
            }
            self.decrement_ref_count(idx);

            // return Handle{index:reference.index, unique:reference.unique, _phantom:PhantomData};
        }
    }
    // pub fn free(&self, handle: IndexedHandle) -> bool {

    //     if handle.index >= self.high_water_mark.get() {
    //         return false;
    //     }
    //     let data = unsafe { self.get_data(handle.index) };
    //     if !data.is_match(handle.unique.get()) {
    //         return false;
    //     }
    //     data.increment_unique();

    //     self.active_count.set(self.active_count.get() - 1);
    //     self.destroyed_count.set(self.destroyed_count.get() + 1);
    //     unsafe {
    //         self.decrement_ref_count(handle.index);
    //     }

    //     return true;
    // }
}

impl<'a, T> Drop for IndexedDataStore<'a, T> {
    fn drop(&mut self) {
        // let t : Nullable<IndexedRef<'a, T>> = Nullable::null();
        //using CAPACITY here is a big, big error - freeing uninitialized memory
        for i in 0..self.high_water_mark.get() {
            unsafe {
                // let data = self.buffer.offset(i as isize).as_mut().unwrap();
                let data = self.get_data(i);
                if data.is_initialized() {
                    ptr::drop_in_place(data.data.as_mut_ptr());
                }
            }
        }
    }
}
// const REF_NULL: u32 = 0xFFFFFFFF;
#[derive(Debug, Hash)]
pub struct IndexedRef<'a, T> {
    index: NonZeroU32,
    // unique : u32,
    _phantom: PhantomData<*mut u8>, //to disable send and sync
    _lifetime: PhantomData<&'a T>,
}
// impl<'a, T> IndexedRef<'a, T> {
//     pub const fn null_const() -> IndexedRef<'a, T> {
//         return IndexedRef {
//             index: REF_NULL,
//             _phantom: PhantomData,
//             _lifetime: PhantomData,
//         };
//     }
// }
impl<'a, T> PartialEq for IndexedRef<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        return self.index == other.index;
    }
}
impl<'a, T> Eq for IndexedRef<'a, T> {}

// impl<'a, T> MaybeNull for IndexedRef<'a, T> {
//     fn is_null(&self) -> bool {
//         return self.index == REF_NULL;
//     }
//     fn null() -> IndexedRef<'a, T> {
//         return IndexedRef {
//             index: REF_NULL,
//             _phantom: PhantomData,
//             _lifetime: PhantomData,
//         };
//     }
//     /// Takes the value out , leaving a null in its place.
//     fn take(&mut self) -> IndexedRef<'a, T> {
//         return IndexedRef {
//             index: core::mem::replace(&mut self.index, REF_NULL),
//             _phantom: PhantomData,
//             _lifetime: PhantomData,
//         };
//     }
//     fn replace(&mut self, new: IndexedRef<'a, T>) -> IndexedRef<'a, T> {
//         return IndexedRef {
//             index: core::mem::replace(&mut self.index, new.index),
//             _phantom: PhantomData,
//             _lifetime: PhantomData,
//         };
//     }
// }

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct IndexedHandle {
    unique: NonZeroU32,
    index: u32,
    _phantom: PhantomData<*mut u8>, //to disable send and sync
}

// impl IndexedHandle {
//     pub fn is_null(self) -> bool {
//         return self.index == REF_NULL;
//     }
// }
#[cfg(test)]
mod test;
