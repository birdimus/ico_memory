use core::cmp::Ordering;
use core::ptr;
use core::borrow::Borrow;
use core::iter::Iterator;
use core::iter::FusedIterator;




/// This is a binary heap which keeps it's data in-place.  
/// This allows a few additional features - such as O(1) retrieval by 'reference', which in turn allows fast key modification 
pub struct BinaryHeap<T : Ord> {
	data : Vec<T>, //T stored contiguously
	heap_index : Vec<u32>, //map from stable address to heap index
	heap : Vec<u32>, // indicies into the underlying data structure.  1-indexed for performance.
	heap_len : u32, // Because we keep the unused indices in the heap around as a free list, we need to track this ourselves
}

impl<T : Ord> BinaryHeap<T> {

	pub fn new() ->BinaryHeap<T>{
		return BinaryHeap{
			data:Vec::new(),
			heap_index:Vec::new(),
			heap:Vec::with_capacity(1),// 1-indexed for performance
			heap_len:0,
		};
	}

	pub fn with_capacity(capacity : u32)->BinaryHeap<T>{
		return BinaryHeap{
			data:Vec::with_capacity(capacity as usize),
			heap_index:Vec::with_capacity(capacity as usize),
			heap:Vec::with_capacity(capacity as usize + 1),// 1-indexed for performance
			heap_len:0,
		};
	}

	pub fn capacity(&self)->u32{
		return self.data.capacity() as u32;
	}
	pub fn len(&self)->u32{
		return self.heap_len;
	}


	/// Parent is i/2 because of 1-indexing.  See CLRS.
	#[inline(always)]
	fn get_parent_index(index : u32)->u32{
		return index >>1;
	}

	/// Right is 2*i + 1 because of 1-indexing.  See CLRS.
	#[inline(always)]
	fn get_right_index(index : u32)->u32{
		return (index <<1)|1;
	}

	/// Right is 2*i because of 1-indexing.  See CLRS.
	#[inline(always)]
	fn get_left_index(index : u32)->u32{
		return (index <<1);
	}

	//TODO: invalidate all references?
	pub fn clear(&mut self){
		self.data.clear();
		self.heap_index.clear();
		self.heap.clear();
		self.heap.push(0xFFFFFFFF); //that first element must be 'null'.
		self.heap_len = 0;
	}


	fn sift_up(&mut self, mut index : u32){

		let data_index = self.heap[index as usize];
		let data : &T = &self.data[data_index as usize];

		while index > 1{
			let parent_index = BinaryHeap::<T>::get_parent_index(index);
			let parent_data_index = self.heap[parent_index as usize];

			//Min Heap
			if(data < &self.data[parent_data_index as usize]){

				// Swap the elements.  Hey it's only integers so swaps are nice and fast
				// We must swap both indices. To maintain the invariant.
				self.heap.swap(index as usize, parent_index as usize);
				self.heap_index.swap(data_index as usize, parent_data_index as usize);

				index = parent_index;
			}
			else{
				return;
			}
		}
	}




	pub fn push(&mut self, item : T){
		let prev_heap_len = self.heap.len() as u32;

		// If the heap is full, we must keep track of the new indicies we've created
		if(self.heap_len == prev_heap_len){
			let data_index = self.data.len() as u32; //This should be heap_len -1.
			// This data will never be moved.
			self.data.push(item);

			// This establishes a relationship between this heap node and the data.
			// It must forever remain true that these two indices point to each other.
			self.heap_index.push(prev_heap_len);
			self.heap.push(data_index);

			self.heap_len +=1;
		}
		else{
			// Add first since we are 1-indexed.
			self.heap_len +=1;
			//Take the last element in the heap.
			let data_index = self.heap[self.heap_len  as usize];
			// That last heap element still refers to a slot.  That slot is empty, so we can reuse it.
			self.data[data_index as usize] = item;
			self.heap_index[data_index as usize] = self.heap_len;
		}

		// Normally we'd use the len-1 (or previous len), but we are 1-indexing.
        self.sift_up( self.heap_len);
	}
}