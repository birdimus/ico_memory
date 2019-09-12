use core::mem::MaybeUninit;
use core::iter::FusedIterator;
use core::iter::Iterator;
use core::ptr;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct UniqueNext{
	unique : u32,
	next : u32,  //this can be used to store flags - and then iterate based on flags (such as 'active')
}

struct Data<T>{
	unique_next : UniqueNext,
	data : MaybeUninit<T>,
}


//TODO: consider using flags here to layer objects for linear layer based queries.

const SLOT_ACTIVE : u32 = (1<<31);
const SLOT_NULL : u32 = !SLOT_ACTIVE;
const INDEX_MASK : u64 = 0x00000000FFFFFFFF;
pub struct IndexedDataStore<T>{

	storage : Vec<Data<T>>,
	free_stack : u32,
	active_count : u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Handle{
	value : u64,//private
}

impl<T> IndexedDataStore<T>{

	pub fn new() ->IndexedDataStore<T>{
		return IndexedDataStore{
			// unique : Vec::new(), 
			storage: Vec::new(),
			free_stack : SLOT_NULL,
			active_count : 0,
		};
	}
	pub fn with_capacity(capacity : usize) ->IndexedDataStore<T>{
		return IndexedDataStore{
			// unique : Vec::new(), 
			storage: Vec::with_capacity(capacity),
			free_stack : SLOT_NULL,
			active_count : 0,
		};
	}
	pub fn high_water_mark(&self)->u32{
		return self.storage.len() as u32;
	}
	pub fn len(&self)->u32{
		return self.active_count as u32;
	}
	pub fn capacity(&self)->u32{
		return self.storage.capacity() as u32;
	}
	pub unsafe fn get_raw(&self, index : u32) ->&T{
		return self.storage[index as usize].data.as_ptr().as_ref().unwrap();
		
	}
	pub unsafe fn get_raw_mut<'a>(&'a mut self, index : u32) ->&'a mut T{
		return self.storage[index as usize].data.as_mut_ptr().as_mut().unwrap();
	}
	pub fn get(&self, handle : Handle) ->Option<&T>{
		let idx = (handle.value & INDEX_MASK) as usize;
		let unique = (handle.value>>32) as u32;
		let unique_next = self.storage[idx].unique_next;
		if unique_next.unique != unique || unique_next.next != SLOT_ACTIVE{
			return None;
		}
		unsafe{
			return self.storage[idx].data.as_ptr().as_ref();
		}
	}
	pub fn get_mut<'a>(&'a mut self, handle : Handle) ->Option<&'a mut T>{
		let idx = (handle.value & INDEX_MASK) as usize;
		let unique = (handle.value>>32) as u32;
		let unique_next = self.storage[idx].unique_next;
		if unique_next.unique != unique || unique_next.next != SLOT_ACTIVE{
			return None;
		}
		unsafe{
			return self.storage[idx].data.as_mut_ptr().as_mut();
		}
	}
	pub fn retain(&mut self, value : T) ->Handle{

		if self.free_stack != SLOT_NULL{
			let result_index = self.free_stack as usize;
			self.storage[result_index].data = MaybeUninit::new(value);
			// self.unique[result_index].unique |= 1;

			let unique_next = self.storage[result_index].unique_next;
			self.free_stack = unique_next.next;
			self.storage[result_index].unique_next.next = SLOT_ACTIVE;
			
			let result : u64 = unique_next.unique as u64;
			self.active_count +=1;
			// println!("reuse");
			return Handle{value:(result<<32) | result_index as u64};

		}
		else{
			let result_index = self.storage.len() as u64;
			// println!("retain");
			self.storage.push(
				Data{
				data:MaybeUninit::new(value),
				unique_next:UniqueNext{unique:1, next:SLOT_ACTIVE },
				}
				);
			self.active_count +=1;
			return Handle{value:(1<<32) | result_index as u64};
		}
	}
	pub fn release(&mut self, handle : Handle) -> Option<T>{
		let idx = (handle.value & INDEX_MASK) as usize;
		let unique = (handle.value>>32) as u32;
		let unique_next = self.storage[idx].unique_next;

		if unique_next.unique != unique || unique_next.next != SLOT_ACTIVE{
			// println!("return failed");
			return None;
		}

		//invalidate all reads
		self.storage[idx].unique_next.unique += 1;//(self.unique[idx].unique&!1) +2;
		
		//drop the value
		// unsafe{
			 // ptr::drop_in_place(self.storage[idx].data.as_mut_ptr());
		// }

		// add slot back to the stack.

		self.storage[idx].unique_next.next = self.free_stack;
		self.free_stack = idx as u32;
		 // println!("stack {} {} {}",self.free_stack, self.storage[idx].unique_next.next, self.storage[idx].unique_next.unique);
		self.active_count -=1;
		unsafe{
		return Some(ptr::read(self.storage[idx].data.as_mut_ptr()));
		}
	}

	pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        return Iter {
            store: self,
            index: 0,
            count: self.active_count,
        };
        
    }
    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
    	let active_count = self.active_count;
        return IterMut {
            store: self,
            index: 0,
            count: active_count,
        };
        
    }
	pub fn clear(&mut self){
		for i in 0..self.storage.len(){
			if self.storage[i].unique_next.next == SLOT_ACTIVE {
				unsafe{
					ptr::drop_in_place(self.storage[i].data.as_mut_ptr());
				}
			}
		}
		self.storage.clear();
		self.free_stack = SLOT_NULL;
		self.active_count =0;
	}
}

impl<T> Drop for IndexedDataStore<T>{
	fn drop(&mut self){
		self.clear();
	}
}
pub struct Iter<'a, T> {
    store : &'a IndexedDataStore<T>,
    count : u32,
    index : u32,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.count == 0 {
            return None;
        };
        let mut i = self.index;
        let hwm = self.store.high_water_mark();
        while i < hwm{
        	if self.store.storage[i as usize].unique_next.next == SLOT_ACTIVE {
        		unsafe{
        			let g = self.store.get_raw(i);
        			self.index = i+1;
        			self.count -= 1;
        			return Some(g);
        		}
        	}
        
        	i+=1;
        }
        //This should never happen.
        return None;
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.count as usize, Some(self.count as usize));
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        return self.count as usize;
    }
}

pub struct IterMut<'a, T> {
    store : &'a mut IndexedDataStore<T>,
    count : u32,
    index : u32,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        if self.count == 0 {
            return None;
        };
        let mut i = self.index;
        let hwm = self.store.high_water_mark();
        while i < hwm{
        	if self.store.storage[i as usize].unique_next.next == SLOT_ACTIVE {
        		
    			self.index = i+1;
    			self.count -= 1;
        			// Borrow checker is being picky about this, so smash it.  We know what we are doing
        		unsafe{
        			return (self.store.get_raw_mut(i) as *mut T).as_mut();
        		}
        	}
        
        	i+=1;
        }
        //This should never happen.
        return None;
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.count as usize, Some(self.count as usize));
    }
}
impl<'a, T> FusedIterator for IterMut<'a, T> {}
impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        return self.count as usize;
    }
}

#[cfg(test)]
mod test;