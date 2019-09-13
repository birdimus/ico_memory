use core::cell::Cell;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;

pub struct IndexedData<T> {
    data: MaybeUninit<T>,
    unique: Cell<u32>,
    ref_count: Cell<u32>,
}
impl<T> IndexedData<T> {
    fn init(&mut self, value: T) {
        self.data = MaybeUninit::new(value);
        self.ref_count.set(1);
        self.unique.set(1);
    }
    fn reinit(&mut self, value: T) {
        self.data = MaybeUninit::new(value);
        self.ref_count.set(1);
        self.set_initialized();
    }
    fn is_initialized(&self) -> bool {
        return (self.unique.get() & 1) == 1;
    }
    fn increment_unique(&mut self) {
        self.unique.set(self.unique.get() + 2);
    }
    fn set_initialized(&mut self) {
        self.unique.set(self.unique.get() | 1);
    }
    fn set_uninitialized(&mut self) {
        self.unique.set(self.unique.get() & !1);
    }
    fn is_match(&self, unique: u32) -> bool {
        return self.unique.get() == unique;
    }
}
const SLOT_NULL: u32 = 0xFFFFFFFF;

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
        if data.ref_count.get() == 0 {
            data.set_uninitialized();
            ptr::drop_in_place(data.data.as_mut_ptr());
            data.ref_count.set(self.free_stack.get());
            self.free_stack.set(index);
            self.destroyed_count.set(self.destroyed_count.get() - 1);
        }
    }

    /// How much of the buffer has ever been used.
    pub fn high_water_mark(&self) -> u32 {
        return self.high_water_mark.get();
    }

    /// How many objects are currently stored and accessible.
    pub fn active(&self) -> u32 {
        return self.active_count.get();
    }

    /// How many objects are currently stored but marked destroyed.
    pub fn destroyed(&self) -> u32 {
        return self.destroyed_count.get();
    }

    /// How much free space.  Capcity - (active + destroyed).
    pub fn available(&self) -> u32 {
        return self.capacity() - self.active() - self.destroyed();
    }

    /// Total fixed capacity.
    pub fn capacity(&self) -> u32 {
        return self.capacity;
    }

    /// Retain a strong reference.  If no reference exists returns None, otherwise, this increments the reference count and returns the reference.  
    /// It is up to the user to manually release() the returned reference when they are finished.
    pub fn retain(&'a self, handle: IndexedHandle) -> Option<IndexedRef<T>> {
        if handle.index >= self.high_water_mark.get() {
            return None;
        }
        unsafe {
            let data = self.get_data(handle.index);
            if !data.is_match(handle.unique) {
                return None;
            }
            data.ref_count.set(data.ref_count.get() + 1);
            return Some(IndexedRef {
                index: handle.index,
                _phantom: PhantomData,
                _lifetime: PhantomData,
            });
            //return data.data.as_ptr().as_ref();
        }
    }
    pub fn get(&'a self, reference: &'a IndexedRef<T>) -> &'a T {
        unsafe {
            let data = self.get_data(reference.index);
            return data.data.as_ptr().as_ref().unwrap();
            // return self.get_raw(reference.index);
        }
    }

    //since we already have a reference, it's safe to get another
    pub fn clone(&'a self, reference: &'a IndexedRef<T>) -> IndexedRef<T> {
        unsafe {
            let data = self.get_data(reference.index);
            data.ref_count.set(data.ref_count.get() + 1);
            return IndexedRef {
                index: reference.index,
                _phantom: PhantomData,
                _lifetime: PhantomData,
            };
            // return self.get_raw(reference.index);
        }
    }

    /// Release a strong reference.  This decrements the reference count.
    pub fn release(&'a self, reference: IndexedRef<T>) {
        unsafe {
            self.decrement_ref_count(reference.index);
            // return Handle{index:reference.index, unique:reference.unique, _phantom:PhantomData};
        }
    }

    pub fn store(&self, value: T) -> IndexedHandle {
        let free = self.free_stack.get();
        if free != SLOT_NULL {
            let data = unsafe { self.get_data(free) };
            self.free_stack.set(data.ref_count.get());

            data.reinit(value);

            let active_count = self.active_count.get();
            self.active_count.set(active_count + 1);
            return IndexedHandle {
                index: free,
                unique: data.unique.get(),
                _phantom: PhantomData,
            };
        } else {
            let hwm = self.high_water_mark.get();
            if hwm < self.capacity {
                self.high_water_mark.set(hwm + 1);
                // println!("index {} {}", result_index, self.buffer as usize);

                let data = unsafe { self.get_data(hwm) };
                data.init(value);

                let active_count = self.active_count.get();
                self.active_count.set(active_count + 1);
                return IndexedHandle {
                    index: hwm,
                    unique: data.unique.get(),
                    _phantom: PhantomData,
                };
            } else {
                #[cfg(any(test, feature = "std"))]
                std::process::abort();
            }
        }
    }
    pub fn free(&self, handle: IndexedHandle) -> bool {
        if handle.index >= self.high_water_mark.get() {
            return false;
        }
        let data = unsafe { self.get_data(handle.index) };
        if !data.is_match(handle.unique) {
            return false;
        }
        data.increment_unique();

        self.active_count.set(self.active_count.get() - 1);
        self.destroyed_count.set(self.destroyed_count.get() + 1);
        unsafe {
            self.decrement_ref_count(handle.index);
        }

        return true;
    }
}

impl<'a, T> Drop for IndexedDataStore<'a, T> {
    fn drop(&mut self) {
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

pub struct IndexedRef<'a, T> {
    index: u32,
    // unique : u32,
    _phantom: PhantomData<*mut u8>, //to disable send and sync
    _lifetime: PhantomData<&'a T>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct IndexedHandle {
    index: u32,
    unique: u32,
    _phantom: PhantomData<*mut u8>, //to disable send and sync
}

#[cfg(test)]
mod test;
