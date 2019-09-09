use crate::sync::IndexSpinlock;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::sync::atomic::AtomicU32;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

// #[allow(dead_code)]
// #[allow(unions_with_drop_fields)]
// pub union Swap<T, U>
// where
//     T: Sized,
// {
//     base: T,
//     other: U,
// }
// // #[allow(dead_code)]
// impl<T, U> Swap<T, U>
// where
//     T: Sized,
// {
//     pub const unsafe fn get(value: T) -> U {
//         return Swap { base: value }.other;
//     }
// }

struct Unique<T> {
    ptr: *const T,           // *const for variance
    _marker: PhantomData<T>, // For the drop checker
}

// Deriving Send and Sync is safe because we are the Unique owners
// of this data. It's like Unique<T> is "just" T.
unsafe impl<T: Send> Send for Unique<T> {}
unsafe impl<T: Sync> Sync for Unique<T> {}

impl<T> Unique<T> {
    pub const fn new(ptr: *mut T) -> Self {
        Unique {
            ptr: ptr,
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr as *mut T
    }
}

/// A MPMC Queue based on Dmitry Vyukov's queue.  
/// However, there is a slight modification where head and tail can be locked, as my implementation of Dmitry's queue failed some tests under peak contention  - and I've opted for a more conservative queue
pub const QUEUE_NULL: usize = 0;
#[repr(C)]
pub struct QueueUsize {
    _cache_pad_0: [u8; 64],
    // buffer: &'a [AtomicUsize],
    buffer: Unique<AtomicUsize>,
    // buffer_ptr : *const AtomicUsize,
    capacity: u32,
    buffer_capacity_mask: u32,
    _cache_pad_1: [u8; 64],
    head: IndexSpinlock,
    _cache_pad_2: [u8; 64],
    tail: IndexSpinlock,
    _cache_pad_3: [u8; 64],
}

impl QueueUsize {
    // const CAPACITY_MASK : u32 = CAPACITY as u32 - 1;

    pub const unsafe fn from_static(slice: *mut AtomicUsize, capacity: usize) -> QueueUsize {
        //pub const fn new(buffer_ptr : *const usize, capacity : usize)->Queue{

        return QueueUsize {
            head: IndexSpinlock::new(0),
            tail: IndexSpinlock::new(0),
            buffer: Unique::new(slice),
            // buffer_ptr : slice.as_ptr() as *const AtomicUsize,
            capacity: capacity as u32,
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
        for i in 0..self.capacity {
            unsafe {
                self.buffer
                    .as_ptr()
                    .offset(i as isize)
                    .as_ref()
                    .unwrap()
                    .store(QUEUE_NULL, Ordering::Relaxed);
            }
        }
        tail.write(0);
        head.write(0);
    }

    pub fn enqueue(&self, value: NonZeroUsize) -> bool {
        let v = value.get();
        debug_assert_ne!(v, QUEUE_NULL);

        let mut tail = self.tail.lock();
        let tail_value = tail.read();

        let storage = unsafe {
            self.buffer
                .as_ptr()
                .offset(tail_value as isize)
                .as_ref()
                .unwrap()
        }; //self.get_storage(tail_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if stored_value != QUEUE_NULL {
            return false;
        }
        storage.store(v, Ordering::Relaxed);
        tail.write(tail_value.wrapping_add(1) & self.buffer_capacity_mask);
        return true;
    }

    pub fn dequeue(&self) -> Option<NonZeroUsize> {
        let mut head = self.head.lock();
        let head_value = head.read();
        let storage = unsafe {
            self.buffer
                .as_ptr()
                .offset(head_value as isize)
                .as_ref()
                .unwrap()
        }; //self.get_storage(head_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if stored_value == QUEUE_NULL {
            return None;
        }
        storage.store(QUEUE_NULL, Ordering::Relaxed);
        head.write(head_value.wrapping_add(1) & self.buffer_capacity_mask);
        unsafe {
            return Some(NonZeroUsize::new_unchecked(stored_value));
        }
    }
}

unsafe impl Send for QueueUsize {}
unsafe impl Sync for QueueUsize {}

#[repr(C)]
pub struct QueueU32 {
    _cache_pad_0: [u8; 64],
    buffer: Unique<AtomicU32>,
    // buffer_ptr : *const AtomicUsize,
    capacity: u32,
    // buffer_ptr : *const AtomicUsize,
    buffer_capacity_mask: u32,
    _cache_pad_1: [u8; 64],
    head: IndexSpinlock,
    _cache_pad_2: [u8; 64],
    tail: IndexSpinlock,
    _cache_pad_3: [u8; 64],
}

pub const QUEUE_U32_NULL: u32 = 0xFFFFFFFF;
impl QueueU32 {
    // const CAPACITY_MASK : u32 = CAPACITY as u32 - 1;

    // #[cfg(any(test, feature = "std"))]
    // pub fn new(capacity: usize) -> QueueU32 {
    //     return QueueU32 {
    //         head: IndexSpinlock::new(0),
    //         tail: IndexSpinlock::new(0),
    //         buffer: Unique::new(slice),
    //         capacity: capacity as u32,
    //         // buffer_ptr : slice.as_ptr() as *const AtomicUsize,
    //         buffer_capacity_mask: capacity as u32 - 1,
    //         _cache_pad_0: [0; 64],
    //         _cache_pad_1: [0; 64],
    //         _cache_pad_2: [0; 64],
    //         _cache_pad_3: [0; 64],
    //     };

    // }
    /// This method is a kludge to work around lack of stable const-generics, const unions, etc.  
    /// It is up to the caller to ensure that the pointer passed in is truly static, and is not mutated externally.
    /// Capacity must be a non-zero power of two.
    pub const unsafe fn from_static(slice: *mut AtomicU32, capacity: usize) -> QueueU32 {
        //pub const fn new(buffer_ptr : *const usize, capacity : usize)->Queue{

        return QueueU32 {
            head: IndexSpinlock::new(0),
            tail: IndexSpinlock::new(0),
            buffer: Unique::new(slice),
            capacity: capacity as u32,
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
        for i in 0..self.capacity {
            unsafe {
                self.buffer
                    .as_ptr()
                    .offset(i as isize)
                    .as_ref()
                    .unwrap()
                    .store(QUEUE_U32_NULL, Ordering::Relaxed);
            }
        }
        tail.write(0);
        head.write(0);
    }

    pub fn enqueue(&self, value: u32) -> bool {
        debug_assert_ne!(value, QUEUE_U32_NULL);

        let mut tail = self.tail.lock();
        let tail_value = tail.read();

        let storage = unsafe {
            self.buffer
                .as_ptr()
                .offset(tail_value as isize)
                .as_ref()
                .unwrap()
        }; //self.get_storage(tail_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if stored_value != QUEUE_U32_NULL {
            return false;
        }
        storage.store(value, Ordering::Relaxed);
        tail.write(tail_value.wrapping_add(1) & self.buffer_capacity_mask);
        return true;
    }

    pub fn dequeue(&self) -> Option<u32> {
        let mut head = self.head.lock();
        let head_value = head.read();
        let storage = unsafe {
            self.buffer
                .as_ptr()
                .offset(head_value as isize)
                .as_ref()
                .unwrap()
        }; //self.get_storage(head_value as usize);
        let stored_value = storage.load(Ordering::Relaxed);
        if stored_value == QUEUE_U32_NULL {
            return None;
        }
        storage.store(QUEUE_U32_NULL, Ordering::Relaxed);
        head.write(head_value.wrapping_add(1) & self.buffer_capacity_mask);

        return Some(stored_value);
    }
}

unsafe impl Send for QueueU32 {}
unsafe impl Sync for QueueU32 {}

#[cfg(test)]
mod test;
