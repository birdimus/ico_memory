use core::cmp::Ordering;
// use core::iter::FusedIterator;
// use core::iter::Iterator;
use core::mem::MaybeUninit;
use core::ptr;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Handle {
    index: u32,
    unique: u32,
}

/// This is a min-heap which keeps it's data in-place.  
/// This allows a few additional features - such as O(1) retrieval by 'reference', which in turn allows fast key modification
pub struct BinaryHeap<T: Ord> {
    data: Vec<MaybeUninit<T>>, //T stored contiguously
    heap_index: Vec<Handle>, //map from stable address to heap index and a unique offset to invalidate old handles
    heap: Vec<u32>, // indicies into the underlying data structure.  1-indexed for performance.
    heap_len: u32, // Because we keep the unused indices in the heap around as a free list, we need to track this ourselves
}

impl<T: Ord> BinaryHeap<T> {
    pub fn new() -> BinaryHeap<T> {
        let mut b = BinaryHeap {
            data: Vec::new(),
            heap_index: Vec::new(),
            heap: Vec::with_capacity(1), // 1-indexed for performance
            heap_len: 0,
        };
        b.heap.push(0xFFFFFFFF);
        return b;
    }

    pub fn with_capacity(capacity: u32) -> BinaryHeap<T> {
        let mut b = BinaryHeap {
            data: Vec::with_capacity(capacity as usize),
            heap_index: Vec::with_capacity(capacity as usize),
            heap: Vec::with_capacity(capacity as usize + 1), // 1-indexed for performance
            heap_len: 0,
        };
        b.heap.push(0xFFFFFFFF);
        return b;
    }

    pub fn capacity(&self) -> u32 {
        return self.data.capacity() as u32;
    }
    pub fn len(&self) -> u32 {
        return self.heap_len;
    }

    /// Parent is i/2 because of 1-indexing.  See CLRS.
    #[inline(always)]
    fn get_parent_index(index: u32) -> u32 {
        return index >> 1;
    }

    /// Right is 2*i + 1 because of 1-indexing.  See CLRS.
    #[inline(always)]
    fn get_right_index(index: u32) -> u32 {
        return (index << 1) | 1;
    }

    /// Right is 2*i because of 1-indexing.  See CLRS.
    #[inline(always)]
    fn get_left_index(index: u32) -> u32 {
        return index << 1;
    }
    #[inline(always)]
    fn get_data(&self, index: u32) -> &T {
        return unsafe { self.data[index as usize].as_ptr().as_ref().unwrap() };
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.heap_index.clear();
        self.heap.clear();
        self.heap.push(0xFFFFFFFF); //the first element must be 'null'.
        self.heap_len = 0;
    }

    fn sift_up(&mut self, mut index: u32) {
        let data_index = self.heap[index as usize];
        let data: &T = unsafe { self.data[data_index as usize].as_ptr().as_ref().unwrap() };

        while index > 1 {
            let parent_index = BinaryHeap::<T>::get_parent_index(index);
            let parent_data_index = self.heap[parent_index as usize];

            //Min Heap
            if data < self.get_data(parent_data_index) {
                // Swap the elements.  Hey it's only integers so swaps are nice and fast
                // We must swap both indices. To maintain the invariant.
                self.heap.swap(index as usize, parent_index as usize);
                self.heap_index
                    .swap(data_index as usize, parent_data_index as usize);

                index = parent_index;
            } else {
                return;
            }
        }
    }

    fn heapify(&mut self, index: u32) {
        let mut best_index = BinaryHeap::<T>::get_left_index(index);
        if best_index > self.heap_len {
            return;
        }

        let mut best_data_index = self.heap[best_index as usize];
        let mut best_data: &T = &self.get_data(best_data_index);

        let right_index = BinaryHeap::<T>::get_right_index(index);
        if right_index <= self.heap_len {
            let right_data_index = self.heap[right_index as usize];
            let right_data: &T = &self.get_data(right_data_index);

            if right_data < best_data {
                best_data = right_data;
                best_index = right_index;
                best_data_index = right_data_index
            }
        }

        let data_index = self.heap[index as usize];
        let data: &T = &self.get_data(data_index);

        if best_data < data {
            self.heap.swap(index as usize, best_index as usize);
            self.heap_index
                .swap(data_index as usize, best_data_index as usize);
        }

        // Tail-recursive.
        self.heapify(best_index);
    }

    fn heap_remove(&mut self, index: u32) -> T {
        // Swap 'n Pop
        let data_index = self.heap[index as usize];
        let last_index = self.heap_len;
        let last_data_index = self.heap[last_index as usize];
        self.heap.swap(index as usize, last_index as usize);
        self.heap_index
            .swap(data_index as usize, last_data_index as usize);
        self.heap_len -= 1;

        //Now we need to restore the heap, starting from the top.
        self.heapify(1);

        // This is optional - but allows us to invalidate the data.
        // If we use unique values, this isn't necessary.
        //self.heap_index[data_index as usize].index = 0xFFFFFFFF;

        // This is a better way to invalidate handles
        self.heap_index[data_index as usize].unique += 1;

        //Just read it out, damnit.
        return unsafe { ptr::read(self.data[data_index as usize].as_mut_ptr()) };
    }

    pub fn push(&mut self, item: T) -> Handle {
        let prev_heap_len = self.data.len() as u32;

        let mut handle: Handle;
        // If the heap is full, we must keep track of the new indicies we've created
        if self.heap_len == prev_heap_len {
            // This data will never be moved.
            self.data.push(MaybeUninit::new(item));
            self.heap_len += 1;

            // This establishes a relationship between this heap node and the data.
            // The heap node MUST always point to the heap_index node.
            handle = Handle {
                index: prev_heap_len,
                unique: 0,
            };
            self.heap_index.push(Handle {
                index: self.heap_len,
                unique: 0,
            });
            self.heap.push(prev_heap_len);
        } else {
            // Add first since we are 1-indexed.
            self.heap_len += 1;

            //Take the last element in the heap.
            let data_index = self.heap[self.heap_len as usize];

            // That last heap element still refers to a slot.  That slot is empty, so we can reuse it.  We already incremented the unique
            self.heap_index[data_index as usize].index = self.heap_len;
            handle = Handle {
                index: data_index,
                unique: self.heap_index[data_index as usize].unique,
            };
            self.data[data_index as usize] = MaybeUninit::new(item);
        }

        // Normally we'd use the len-1 (or previous len), but we are 1-indexing.
        self.sift_up(self.heap_len);
        return handle;
    }

    /// O(1) get element in the binary-heap.
    pub fn get(&self, handle: Handle) -> Option<&T> {
        if handle.index >= self.heap_index.len() as u32 {
            return None;
        }
        let h = self.heap_index[handle.index as usize];
        if h.unique != handle.unique {
            return None;
        }
        return unsafe { self.data[handle.index as usize].as_ptr().as_ref() };
    }

    /// Replaces the element in the binary heap, and moves it to it's correct position.
    pub fn replace(&mut self, handle: Handle, item: T) -> bool {
        if handle.index >= self.heap_index.len() as u32 {
            return false;
        }
        let mut h = self.heap_index[handle.index as usize];
        if h.unique != handle.unique {
            return false;
        }

        let ordering = item.cmp(self.get_data(handle.index));

        self.data[handle.index as usize] = MaybeUninit::new(item);

        match ordering {
            Ordering::Less => self.sift_up(h.index),
            Ordering::Greater => self.heapify(h.index),
            Ordering::Equal => {}
        }

        return true;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.heap_len == 0 {
            return None;
        }
        return Some(self.heap_remove(1));
    }
}

#[cfg(test)]
mod test;
