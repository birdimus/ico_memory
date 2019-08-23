//concurrent hashtable
// N buckets
// Each bucket is a tree protected by a rwlock\
// Add and Remove alloc / dealloc

// concurrent linked list?
// each element protected by rwlock
// hand over hand locking
// Add and Remove alloc / dealloc

use core::cmp::Ord;
use core::cmp::Ordering;
use core::ptr;
use core::borrow::Borrow;
use core::iter::Iterator;
extern crate alloc;

pub struct Entry<K: Ord, V>{
	key: K,
    value: V,
}

pub struct AVLNode<K: Ord, V> {
    entry : Entry<K,V>,
    parent: *mut AVLNode<K, V>,
    left: *mut AVLNode<K, V>,//Option<Box<AVLNode<K, V>>>,
    right: *mut AVLNode<K, V>,//Option<Box<AVLNode<K, V>>>,
    height: u8,
}

impl<K: Ord, V> AVLNode<K, V>{


	pub fn key(&self) ->&K{
		return &self.entry.key;
	}
	pub fn value(&self) ->&V{
		return &self.entry.value;
	}
	pub fn value_mut(&mut self) ->&mut V{
		return &mut self.entry.value;
	}

	fn new(key : K, value : V) ->AVLNode<K, V>{
		return AVLNode{
			height:1, 
			right:ptr::null_mut(),//None,
			left:ptr::null_mut(),//None,
			parent:ptr::null_mut(),
			entry:Entry{key:key, value:value},
		};
	}

	pub(crate) fn get_height(node: Option<&AVLNode<K, V>>)->u8{
		match node{
			Some(n)=>{return n.height;},
			None=>{return 0;},
		}
	}
	fn max_height(& self) ->u8{
		
		let right_or = unsafe{self.right.as_ref()};
		let right_height = AVLNode::get_height(right_or);

		let left_or = unsafe{self.left.as_ref()};
		let left_height = AVLNode::get_height(left_or);
		if left_height > right_height {return left_height;} else {return right_height;}
	}

	fn recalculate_height(&mut self){
		self.height = 1 + self.max_height();
	}

	pub(crate) fn balance(& self)->i32{
		let right_or = unsafe{self.right.as_ref()};
		let right_height = AVLNode::get_height(right_or) as i32;

		let left_or = unsafe{self.left.as_ref()};
		let left_height = AVLNode::get_height(left_or)  as i32;
		return left_height - right_height
	}

	fn max_child(&self) -> &AVLNode<K, V>{
		let mut node = self;
		while !node.right.is_null() {
			node = unsafe{node.right.as_ref().unwrap()};
		}
		return node;
	}

	fn max_child_mut(&mut self) -> &mut AVLNode<K, V>{
		let mut node = self;
		while !node.right.is_null() {
			node = unsafe{node.right.as_mut().unwrap()};
		}
		return node;
	}

	fn min_child(&self) -> &AVLNode<K, V>{
		let mut node = self;
		while !node.left.is_null() {
			node = unsafe{node.left.as_ref().unwrap()};
		}
		return node;
	}

	fn min_child_mut(&mut self) -> &mut AVLNode<K, V>{
		let mut node = self;
		while !node.left.is_null() {
			node = unsafe{node.left.as_mut().unwrap()};
		}
		return node;
	}

	// pub fn next(&self) ->Option<& AVLNode<K, V> >{
	// 	if self.right.is_some(){
	// 		return Some(self.right.as_ref().unwrap().min_child());
	// 	}

	// 	let mut node = self;
	// 	let mut p = unsafe{node.parent.as_ref()};
	// 	while p.is_some() {
	// 		let parent = p.unwrap();
	// 		if ptr::eq(node,parent.left.as_ref().unwrap().as_ref()){
	// 			return Some(parent);		
	// 		}
	// 		node = parent;
	// 		p = unsafe{node.parent.as_ref()};
	// 	}
	// 	return p;
	// }
	
	// pub fn next_mut(&mut self) ->Option<&mut AVLNode<K, V> >{
	// 	if self.right.is_some(){
	// 		return Some(self.right.as_mut().unwrap().min_child_mut());
	// 	}

	// 	let mut node = self;
	// 	let mut p = unsafe{node.parent.as_mut()};
	// 	while p.is_some() {
	// 		let parent = p.unwrap();
	// 		if ptr::eq(node,parent.left.as_mut().unwrap().as_mut()){
	// 			return Some(parent);		
	// 		}
	// 		node = parent;
	// 		p = unsafe{node.parent.as_mut()};
	// 	}
	// 	return p;
	// }
	// pub fn previous(&self)->Option<& AVLNode<K, V> >{
	// 	if self.left.is_some(){
	// 		return Some(self.left.as_ref().unwrap().max_child());
	// 	}

	// 	let mut node = self;
	// 	let mut p = unsafe{node.parent.as_ref()};
	// 	while p.is_some() {
	// 		let parent = p.unwrap();
	// 		if ptr::eq(node,parent.right.as_ref().unwrap().as_ref()){
	// 			return Some(parent);		
	// 		}
	// 		node = parent;
	// 		p = unsafe{node.parent.as_ref()};
	// 	}
	// 	return p;
	// }
	// pub fn previous_mut(&mut self)->Option<&mut AVLNode<K, V> >{
	// 	return None;
	// }
	// pub fn entry_mut(&mut self) ->&mut Entry<K,V>{
	// 	return &mut self.entry;
	// }
	
	// pub fn extract(&mut self) ->(&mut Entry<K,V>, Option<&mut AVLNode<K, V> >){
	// 	return (&mut self.entry, self.left.as_mut().map(|node| &mut **node)   );
	// }
}


pub struct AVLTree<K: Ord, V>{
    root: *mut AVLNode<K, V>,//Option<Box<AVLNode<K, V>>>,
    first: *mut AVLNode<K, V>,
    last: *mut AVLNode<K, V>,
    count: u32,
}

impl<K: Ord, V> AVLTree<K, V>{


	pub fn new()->AVLTree<K, V>{
		return AVLTree{
			root: ptr::null_mut(), 
			first: ptr::null_mut(),
			last: ptr::null_mut(),
			count: 0,
		}
	}

	pub fn first(){

	}
	pub fn last(){

	}
	pub fn count(&self) ->u32{
		return self.count;
	}

	fn rotate_right(&mut self, base : &mut AVLNode<K, V>, left : &mut AVLNode<K, V>){

		let left_r = left.right;
		left.right = base;
		base.left = left_r;
		if(!left_r.is_null()){
			unsafe{left_r.as_mut().unwrap().parent = base;}
		}
		
		let grandparent = base.parent;
		base.parent = left;
		left.parent = grandparent;
		if(!grandparent.is_null()){
			let gp_node = unsafe{grandparent.as_mut().unwrap()};
			if(gp_node.left == base){
				gp_node.left = left;
			}
			else{
				gp_node.right = left;
			}
		}
		else{
			self.root = left;
		}
		base.recalculate_height();
		left.recalculate_height();
		
	}
	fn rotate_left(&mut self, base : &mut AVLNode<K, V>, right : &mut AVLNode<K, V>){

		let right_l = right.left;
		right.left = base;
		base.right = right_l;
		if(!right_l.is_null()){
			unsafe{right_l.as_mut().unwrap().parent = base;}
		}
		
		let grandparent = base.parent;
		base.parent = right;
		right.parent = grandparent;
		if(!grandparent.is_null()){
			let gp_node = unsafe{grandparent.as_mut().unwrap()};
			if(gp_node.left == base){
				gp_node.left = right;
			}
			else{
				gp_node.right = right;
			}
		}
		else{
			self.root = right;
		}
		base.recalculate_height();
		right.recalculate_height();
	}

	fn rebalance(&mut self, from_node : &mut AVLNode<K, V>){
		
		let mut node = from_node;
		loop{

			node.recalculate_height();
			let balance = node.balance();
			if balance > 1{
				let left_node = unsafe{node.left.as_mut().unwrap()};
				if left_node.balance() > 0{
					self.rotate_right(node, left_node);
					node = left_node; //this is the new position.
				}
				else{
					let left_right = unsafe{left_node.right.as_mut().unwrap()};
					self.rotate_left(left_node, left_right);
					self.rotate_right(node, left_right);
					node = left_right;
				}
				
			}
			else if balance < -1{
				let right_node = unsafe{node.right.as_mut().unwrap()};
				if right_node.balance() < 0{
					self.rotate_left(node, right_node);
					node = right_node; //this is the new position.
				}
				else{
					let right_left = unsafe{right_node.left.as_mut().unwrap()};
					self.rotate_right(right_node, right_left);
					self.rotate_left(node, right_left);
					node = right_left;
				}

			}

			if node.parent.is_null(){
				return;
			}
			else{
				node = unsafe{node.parent.as_mut().unwrap()};
			}
		}
	}
	
	pub fn insert(&mut self, key : K, value : V) ->bool{
		
		if self.root.is_null(){
			
			let mut raw_node : *mut AVLNode<K, V> = ptr::null_mut();
			unsafe{
				let layout = core::alloc::Layout::from_size_align_unchecked(core::mem::size_of::<AVLNode<K, V>>(), core::mem::align_of::<AVLNode<K, V>>());
				raw_node = alloc::alloc::alloc(layout) as *mut AVLNode<K, V>;
				*raw_node = AVLNode::new(key, value);
			}
			self.count += 1;
			self.first = raw_node;
			self.last = raw_node;
			self.root = raw_node;
			return true;
		}
		
		let mut parent = ptr::null_mut();
		let mut target = self.root;
		let mut is_left : bool = false;
		while let Some(node) = unsafe{target.as_ref()}{
			parent = target;
			let ord = key.cmp(&node.entry.key);
				match ord{
    			Ordering::Less=>{target = node.left; is_left = true;},
    			Ordering::Greater=>{target = node.right; is_left = false;},
    			Ordering::Equal=>{

    				return false;},
    		}
		}
		

		let mut raw_node : *mut AVLNode<K, V> = ptr::null_mut();
		unsafe{
			let layout = core::alloc::Layout::from_size_align_unchecked(core::mem::size_of::<AVLNode<K, V>>(), core::mem::align_of::<AVLNode<K, V>>());
			raw_node = alloc::alloc::alloc(layout) as *mut AVLNode<K, V>;
			*raw_node = AVLNode::new(key, value);
		}
		self.count += 1;
		let mut new_node = unsafe{raw_node.as_mut().unwrap()};
		let mut new_node_parent = unsafe{parent.as_mut().unwrap()};
		new_node.parent = parent;
		if(is_left){
			new_node_parent.left = raw_node;
			let smallest = unsafe{self.first.as_ref().unwrap()};
			let ord = new_node.entry.key.cmp(&smallest.entry.key);
			if ord == Ordering::Less{
				self.first = raw_node;
			}
		}
		else{
			new_node_parent.right = raw_node;
			let largest = unsafe{self.last.as_ref().unwrap()};
			let ord = new_node.entry.key.cmp(&largest.entry.key);
			if ord == Ordering::Greater{
				self.last = raw_node;
			}
		}	
		self.rebalance(new_node_parent);

  		return true;
	}

	pub fn entry<Q>(&self, search_key : &Q) -> Option<&AVLNode<K, V> >
	where
    K: Borrow<Q>,
    Q: Ord + ?Sized{

    	let mut target = self.root;
    	while let Some(node) = unsafe{target.as_ref()}{
    		let ord = search_key.cmp(node.entry.key.borrow());
    		match ord{
    			Ordering::Less=>{target = node.left;},
    			Ordering::Greater=>{target = node.right;},
    			Ordering::Equal=>{
    				return Some(node);
    			},
    		}
    		
    	}
    	return None;
	}

	pub fn entry_mut<Q>(&mut self, search_key : &Q) -> Option<&mut AVLNode<K, V> >
	where
    K: Borrow<Q>,
    Q: Ord + ?Sized{

    	let mut target = self.root;
    	while let Some(node) = unsafe{target.as_mut()}{
    		let ord = search_key.cmp(node.entry.key.borrow());
    		match ord{
    			Ordering::Less=>{target = node.left;},
    			Ordering::Greater=>{target = node.right;},
    			Ordering::Equal=>{
    				return Some(node);
    			},
    		}
    		
    	}
    	return None;
	}

	pub fn contains<Q>(&self, search_key : &Q) -> bool
	where
    K: Borrow<Q>,
    Q: Ord + ?Sized{

    	return self.entry(search_key).is_some();
	}

	pub fn get<Q>(&self, search_key : &Q) -> Option<&V>
	where
    K: Borrow<Q>,
    Q: Ord + ?Sized{

    	return self.entry(search_key).map(|node| node.value());
	}

	pub fn get_mut<Q>(&mut self, search_key : &Q) -> Option<&mut V>
	where
    K: Borrow<Q>,
    Q: Ord + ?Sized{

    	return self.entry_mut(search_key).map(|node| node.value_mut());
	}
	// pub fn iter(&self) -> Iter<'_, K,V> {
 //        Iter { next: self.root.as_ref().map(|node| &**node) }
 //    }

 //    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
 //        IterMut { next: self.root.as_mut().map(|node| &mut **node) }
 //    }

}

// pub struct IntoIter<T>(List<T>);
// pub struct Iter<'a, K: Ord, V> {
//     next: Option<&'a AVLNode<K, V> >,
// }

// pub struct IterMut<'a, K: Ord, V> {
//     next: Option<&'a mut AVLNode<K, V> >,
// }
// pub trait ForwardBackwardIterator : Iterator {
//     fn prev(&mut self) -> Option<Self::Item>;
// }
// pub trait ForwardBackwardIteratorMut : Iterator {
//     fn prev(&mut self) -> Option<Self::Item>;
// }

// impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
//     type Item = &'a Entry<K,V>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.next.map(|node| {
//             self.next = node.next();//.as_ref().map(|node| &**node);
//             return &node.entry;
//         })
//     }
// }
// impl<'a, K: Ord, V> ForwardBackwardIterator for Iter<'a, K, V> {

//     fn prev(&mut self) -> Option<Self::Item> {
//         self.next.map(|node| {
//             self.next = node.previous();//.as_ref().map(|node| &**node);
//             return &node.entry;
//         })
//     }
// }

// impl<'a, K: Ord, V> Iterator for IterMut<'a, K, V> {
//     type Item = &'a mut Entry<K,V>;

//     fn next(&mut self) -> Option<Self::Item> {

//     	let t = self.next.take();
//     	if(t.is_none()){return None;}
//     	let q = t.unwrap();
//     	//This works around the borrow checker being a bit clueless with regard to borrowing multiple parts of a struct.

//     	self.next = q.next_mut();


//     	return None;//Some( borrow_hack.0);

//     }
// }
// impl<'a, K: Ord, V> ForwardBackwardIteratorMut for IterMut<'a, K, V> {

//     fn prev(&mut self) -> Option<Self::Item> {
//     	let t = self.next.take();
//     	if(t.is_none()){return None;}
//     	let q = t.unwrap();
//     	//This works around the borrow checker being a bit clueless with regard to borrowing multiple parts of a struct.
//     	let borrow_hack = q.extract();

//     	self.next = borrow_hack.1;
//     	return Some( borrow_hack.0);

//     }
// }

#[cfg(test)]
mod test;
