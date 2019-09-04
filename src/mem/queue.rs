use crate::sync::index_lock::IndexSpinlock;
use core::num::NonZeroUsize;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering;

/// A MPMC Queue based on Dmitry Vyukov's queue.  
/// However, there is a slight modification where head and tail can be locked, as my implementation of Dmitry's queue failed some tests under peak contention  - and I've opted for a more conservative queue
pub const QUEUE_NULL : usize = 0;
#[repr(C)]
pub struct Queue<'a> {
    _cache_pad_0: [u8; 64],
    buffer: &'a [AtomicUsize],
    // buffer_ptr : *const AtomicUsize,
    buffer_capacity_mask: u32,
    _cache_pad_1: [u8; 64],
    head: IndexSpinlock,
    _cache_pad_2: [u8; 64],
    tail: IndexSpinlock,
    _cache_pad_3: [u8; 64],
}

// #[allow(dead_code)]
#[allow(unions_with_drop_fields)]
pub union Swap<T, U>
where
    T: Sized,
{
    base: T,
    other: U,
}
// #[allow(dead_code)]
impl<T, U> Swap<T, U>
where
    T: Sized,
{
    pub const unsafe fn get(value: T) -> U {
        return Swap { base: value }.other;
    }
}

impl<'a> Queue<'a> {
    // const CAPACITY_MASK : u32 = CAPACITY as u32 - 1;

    pub const fn new(slice: &'a [AtomicUsize], capacity: usize) -> Queue<'a> {
        //pub const fn new(buffer_ptr : *const usize, capacity : usize)->Queue{

        return Queue::<'a> {
            head: IndexSpinlock::new(0),
            tail: IndexSpinlock::new(0),
            buffer: slice,
            // buffer_ptr : slice.as_ptr() as *const AtomicUsize,
            buffer_capacity_mask: capacity as u32 - 1,
            _cache_pad_0: [0; 64],
            _cache_pad_1: [0; 64],
            _cache_pad_2: [0; 64],
            _cache_pad_3: [0; 64],
        };
    }
    pub fn clear(&self) {
        let mut tail = self.tail.lock();
        let mut head = self.head.lock();
        for i in 0..self.buffer.len() {
            self.buffer[i].store(0, Ordering::Relaxed);
        }
        tail.write(0);
        head.write(0);
    }


    pub fn enqueue(&self, value: NonZeroUsize) -> bool {
        let v = value.get();
        debug_assert_ne!(v, QUEUE_NULL);

        let mut tail = self.tail.lock();
        let tail_value = tail.read();

        let storage = unsafe { self.buffer.get_unchecked(tail_value as usize) }; //self.get_storage(tail_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if (stored_value != QUEUE_NULL) {
            return false;
        }
        storage.store(v, Ordering::Relaxed);
        tail.write(tail_value.wrapping_add(1) & self.buffer_capacity_mask);
        return true;
    }

    pub fn dequeue(&self) -> Option<NonZeroUsize> {
        let mut head = self.head.lock();
        let head_value = head.read();
        let storage = unsafe { self.buffer.get_unchecked(head_value as usize) }; //self.get_storage(head_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if (stored_value == QUEUE_NULL) {
            return None;
        }
        storage.store(QUEUE_NULL, Ordering::Relaxed);
        head.write(head_value.wrapping_add(1) & self.buffer_capacity_mask);
        unsafe {
            return Some(NonZeroUsize::new_unchecked(stored_value));
        }
    }
}

unsafe impl<'a> Send for Queue<'a> {}
unsafe impl<'a> Sync for Queue<'a> {}


#[repr(C)]
pub struct Queue32<'a> {
    _cache_pad_0: [u8; 64],
    buffer: &'a [AtomicU32],
    // buffer_ptr : *const AtomicUsize,
    buffer_capacity_mask: u32,
    _cache_pad_1: [u8; 64],
    head: IndexSpinlock,
    _cache_pad_2: [u8; 64],
    tail: IndexSpinlock,
    _cache_pad_3: [u8; 64],
}


pub const QUEUE32_NULL : u32 = 0xFFFFFFFF;
impl<'a> Queue32<'a> {
    // const CAPACITY_MASK : u32 = CAPACITY as u32 - 1;

    pub const fn new(slice: &'a [AtomicU32], capacity: usize) -> Queue32<'a> {
        //pub const fn new(buffer_ptr : *const usize, capacity : usize)->Queue{

        return Queue32::<'a> {
            head: IndexSpinlock::new(0),
            tail: IndexSpinlock::new(0),
            buffer: slice,
            // buffer_ptr : slice.as_ptr() as *const AtomicUsize,
            buffer_capacity_mask: capacity as u32 - 1,
            _cache_pad_0: [0; 64],
            _cache_pad_1: [0; 64],
            _cache_pad_2: [0; 64],
            _cache_pad_3: [0; 64],
        };
    }
    pub fn clear(&self) {
        let mut tail = self.tail.lock();
        let mut head = self.head.lock();
        for i in 0..self.buffer.len() {
            self.buffer[i].store(0, Ordering::Relaxed);
        }
        tail.write(0);
        head.write(0);
    }


    pub fn enqueue(&self, value: u32) -> bool {
        debug_assert_ne!(value, QUEUE32_NULL);

        let mut tail = self.tail.lock();
        let tail_value = tail.read();

        let storage = unsafe { self.buffer.get_unchecked(tail_value as usize) }; //self.get_storage(tail_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if (stored_value != QUEUE32_NULL) {
            return false;
        }
        storage.store(value, Ordering::Relaxed);
        tail.write(tail_value.wrapping_add(1) & self.buffer_capacity_mask);
        return true;
    }

    pub fn dequeue(&self) -> Option<u32> {
        let mut head = self.head.lock();
        let head_value = head.read();
        let storage = unsafe { self.buffer.get_unchecked(head_value as usize) }; //self.get_storage(head_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if (stored_value == QUEUE32_NULL) {
            return None;
        }
        storage.store(QUEUE32_NULL, Ordering::Relaxed);
        head.write(head_value.wrapping_add(1) & self.buffer_capacity_mask);
        unsafe {
            return Some(stored_value);
        }
    }
}

unsafe impl<'a> Send for Queue32<'a> {}
unsafe impl<'a> Sync for Queue32<'a> {}


#[cfg(test)]
mod test;
