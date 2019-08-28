use core::borrow::Borrow;
use core::cmp::Ord;
use core::cmp::Ordering;
use core::iter::FusedIterator;
use core::iter::Iterator;
use core::ptr;
extern crate alloc;

pub struct Entry<K: Ord, V> {
    key: K,
    value: V,
}
impl<K: Ord, V> Entry<K, V> {
    pub fn key(&self) -> &K {
        return &self.key;
    }
    pub fn value(&self) -> &V {
        return &self.value;
    }
    pub fn set_value(&mut self, v: V) {
        return self.value = v;
    }
}
pub struct AVLNode<K: Ord, V> {
    entry: Entry<K, V>,
    parent: *mut AVLNode<K, V>,
    left: *mut AVLNode<K, V>,  //Option<Box<AVLNode<K, V>>>,
    right: *mut AVLNode<K, V>, //Option<Box<AVLNode<K, V>>>,
    height: u8,
}

impl<K: Ord, V> AVLNode<K, V> {
    pub fn key(&self) -> &K {
        return &self.entry.key;
    }
    pub fn value(&self) -> &V {
        return &self.entry.value;
    }
    pub fn value_mut(&mut self) -> &mut V {
        return &mut self.entry.value;
    }

    fn new(key: K, value: V) -> AVLNode<K, V> {
        return AVLNode {
            height: 1,
            right: ptr::null_mut(), //None,
            left: ptr::null_mut(),  //None,
            parent: ptr::null_mut(),
            entry: Entry {
                key: key,
                value: value,
            },
        };
    }

    pub(crate) fn get_height(node: Option<&AVLNode<K, V>>) -> u8 {
        match node {
            Some(n) => {
                return n.height;
            }
            None => {
                return 0;
            }
        }
    }
    fn max_height(&self) -> u8 {
        let right_or = unsafe { self.right.as_ref() };
        let right_height = AVLNode::get_height(right_or);

        let left_or = unsafe { self.left.as_ref() };
        let left_height = AVLNode::get_height(left_or);
        if left_height > right_height {
            return left_height;
        } else {
            return right_height;
        }
    }

    fn recalculate_height(&mut self) {
        self.height = 1 + self.max_height();
    }

    pub(crate) fn balance(&self) -> i32 {
        let right_or = unsafe { self.right.as_ref() };
        let right_height = AVLNode::get_height(right_or) as i32;

        let left_or = unsafe { self.left.as_ref() };
        let left_height = AVLNode::get_height(left_or) as i32;
        return left_height - right_height;
    }

    fn max_child(&self) -> &AVLNode<K, V> {
        let mut node = self;
        while !node.right.is_null() {
            node = unsafe { node.right.as_ref().unwrap() };
        }
        return node;
    }

    fn max_child_mut(&mut self) -> &mut AVLNode<K, V> {
        let mut node = self;
        while !node.right.is_null() {
            node = unsafe { node.right.as_mut().unwrap() };
        }
        return node;
    }

    fn min_child(&self) -> &AVLNode<K, V> {
        let mut node = self;
        while !node.left.is_null() {
            node = unsafe { node.left.as_ref().unwrap() };
        }
        return node;
    }

    fn min_child_mut(&mut self) -> &mut AVLNode<K, V> {
        let mut node = self;
        while !node.left.is_null() {
            node = unsafe { node.left.as_mut().unwrap() };
        }
        return node;
    }

    fn successor(&self) -> Option<&AVLNode<K, V>> {
        if !self.right.is_null() {
            unsafe {
                return Some(self.right.as_ref().unwrap().min_child());
            }
        }

        let mut node = self;
        let mut p = unsafe { node.parent.as_ref() };
        while p.is_some() {
            let parent = p.unwrap();
            if ptr::eq(node, parent.left) {
                return Some(parent);
            }
            node = parent;
            p = unsafe { node.parent.as_ref() };
        }
        return p;
    }

    pub fn predecessor(&self) -> Option<&AVLNode<K, V>> {
        if !self.left.is_null() {
            unsafe {
                return Some(self.left.as_ref().unwrap().max_child());
            }
        }

        let mut node = self;
        let mut p = unsafe { node.parent.as_ref() };
        while p.is_some() {
            let parent = p.unwrap();
            if ptr::eq(node, parent.right) {
                return Some(parent);
            }
            node = parent;
            p = unsafe { node.parent.as_ref() };
        }
        return p;
    }
    fn successor_mut(&mut self) -> Option<&mut AVLNode<K, V>> {
        if !self.right.is_null() {
            unsafe {
                return Some(self.right.as_mut().unwrap().min_child_mut());
            }
        }

        let mut node = self;
        let mut p = unsafe { node.parent.as_mut() };
        while p.is_some() {
            let parent = p.unwrap();
            if ptr::eq(node, parent.left) {
                return Some(parent);
            }
            node = parent;
            p = unsafe { node.parent.as_mut() };
        }
        return p;
    }

    pub fn predecessor_mut(&mut self) -> Option<&mut AVLNode<K, V>> {
        if !self.left.is_null() {
            unsafe {
                return Some(self.left.as_mut().unwrap().max_child_mut());
            }
        }

        let mut node = self;
        let mut p = unsafe { node.parent.as_mut() };
        while p.is_some() {
            let parent = p.unwrap();
            if ptr::eq(node, parent.right) {
                return Some(parent);
            }
            node = parent;
            p = unsafe { node.parent.as_mut() };
        }
        return p;
    }

    // pub fn previous_mut(&mut self)->Option<&mut AVLNode<K, V> >{
    // 	return None;
    // }
    // pub fn entry_mut(&mut self) ->&mut Entry<K,V>{
    // 	return &mut self.entry;
    // }
}

pub struct AVLTree<K: Ord, V> {
    root: *mut AVLNode<K, V>, //Option<Box<AVLNode<K, V>>>,
    first: *mut AVLNode<K, V>,
    last: *mut AVLNode<K, V>,
    count: u32,
}

impl<K: Ord, V> AVLTree<K, V> {
    pub fn new() -> AVLTree<K, V> {
        return AVLTree {
            root: ptr::null_mut(),
            first: ptr::null_mut(),
            last: ptr::null_mut(),
            count: 0,
        };
    }

    // pub fn first(){

    // }
    // pub fn last(){

    // }
    pub fn len(&self) -> u32 {
        return self.count;
    }

    fn transplant(&mut self, from: &mut AVLNode<K, V>, to: *mut AVLNode<K, V>) {
        if (from.parent.is_null()) {
            self.root = to;
        } else {
            let parent_node = unsafe { from.parent.as_mut().unwrap() };
            if (from as *mut AVLNode<K, V> == parent_node.left) {
                parent_node.left = to;
            } else {
                parent_node.right = to;
            }
        }
        let to_node = unsafe { to.as_mut() };
        if (to_node.is_some()) {
            to_node.unwrap().parent = from.parent;
        }
    }

    fn rotate_right(&mut self, base: &mut AVLNode<K, V>, left: &mut AVLNode<K, V>) {
        let left_r = left.right;
        left.right = base;
        base.left = left_r;
        if (!left_r.is_null()) {
            unsafe {
                left_r.as_mut().unwrap().parent = base;
            }
        }

        let grandparent = base.parent;
        base.parent = left;
        left.parent = grandparent;
        if (!grandparent.is_null()) {
            let gp_node = unsafe { grandparent.as_mut().unwrap() };
            if (gp_node.left == base) {
                gp_node.left = left;
            } else {
                gp_node.right = left;
            }
        } else {
            self.root = left;
        }
        base.recalculate_height();
        left.recalculate_height();
    }
    fn rotate_left(&mut self, base: &mut AVLNode<K, V>, right: &mut AVLNode<K, V>) {
        let right_l = right.left;
        right.left = base;
        base.right = right_l;
        if (!right_l.is_null()) {
            unsafe {
                right_l.as_mut().unwrap().parent = base;
            }
        }

        let grandparent = base.parent;
        base.parent = right;
        right.parent = grandparent;
        if (!grandparent.is_null()) {
            let gp_node = unsafe { grandparent.as_mut().unwrap() };
            if (gp_node.left == base) {
                gp_node.left = right;
            } else {
                gp_node.right = right;
            }
        } else {
            self.root = right;
        }
        base.recalculate_height();
        right.recalculate_height();
    }

    fn rebalance(&mut self, from_node: &mut AVLNode<K, V>) {
        let mut node = from_node;
        loop {
            node.recalculate_height();
            let balance = node.balance();
            if balance > 1 {
                let left_node = unsafe { node.left.as_mut().unwrap() };
                if left_node.balance() > 0 {
                    self.rotate_right(node, left_node);
                    node = left_node; //this is the new position.
                } else {
                    let left_right = unsafe { left_node.right.as_mut().unwrap() };
                    self.rotate_left(left_node, left_right);
                    self.rotate_right(node, left_right);
                    node = left_right;
                }
            } else if balance < -1 {
                let right_node = unsafe { node.right.as_mut().unwrap() };
                if right_node.balance() < 0 {
                    self.rotate_left(node, right_node);
                    node = right_node; //this is the new position.
                } else {
                    let right_left = unsafe { right_node.left.as_mut().unwrap() };
                    self.rotate_right(right_node, right_left);
                    self.rotate_left(node, right_left);
                    node = right_left;
                }
            }

            if node.parent.is_null() {
                return;
            } else {
                node = unsafe { node.parent.as_mut().unwrap() };
            }
        }
    }

    fn remove_node(&mut self, node: &mut AVLNode<K, V>) -> Entry<K, V> {
        let mut parent = ptr::null_mut();
        if (node.left.is_null()) {
            //since this node has no left children, it could be the min node.
            if self.first == node {
                match node.successor_mut() {
                    Some(next) => self.first = next,
                    None => self.first = ptr::null_mut(),
                }
            }
            parent = node.parent;
            self.transplant(node, node.right);
        //return parent;
        } else if (node.right.is_null()) {
            if self.last == node {
                match node.predecessor_mut() {
                    Some(prev) => self.last = prev,
                    None => self.last = ptr::null_mut(),
                }
            }

            parent = node.parent;
            self.transplant(node, node.left);
        //return parent;
        } else {
            let right_node = unsafe { node.right.as_mut().unwrap() };
            let successor = right_node.min_child_mut(); //the leftmost node
            if (!successor.right.is_null()) {
                let s_right = unsafe { successor.right.as_mut().unwrap() };
                self.transplant(successor, s_right);
            }
            self.transplant(node, successor);
            successor.left = node.left;
            let left_node = unsafe { node.left.as_mut().unwrap() };
            left_node.parent = successor;
            // this is overly conservative when the successor is deep (one extra rebalance node)
            // but catches the case when the
            // successor is the right child of the original node.
            // the branchlessness is probably worth the cost of the additional additions.
            parent = successor;
        }

        //now rebalance from the parent node.
        if !parent.is_null() {
            self.rebalance(unsafe { parent.as_mut().unwrap() })
        }

        self.count -= 1;
        // This box immediately goes out of scope - but we move the contents out, deferring any drop.
        let drop_box: Box<AVLNode<K, V>> = unsafe { Box::from_raw(node as *mut AVLNode<K, V>) };
        return drop_box.entry;
    }

    pub fn insert(&mut self, key: K, value: V) -> bool {
        if self.root.is_null() {
            let raw_node = Box::into_raw(Box::new(AVLNode::new(key, value)));

            self.count += 1;
            self.first = raw_node;
            self.last = raw_node;
            self.root = raw_node;
            return true;
        }

        let mut parent = ptr::null_mut();
        let mut target = self.root;
        let mut is_left: bool = false;
        while let Some(node) = unsafe { target.as_ref() } {
            parent = target;
            let ord = key.cmp(&node.entry.key);
            match ord {
                Ordering::Less => {
                    target = node.left;
                    is_left = true;
                }
                Ordering::Greater => {
                    target = node.right;
                    is_left = false;
                }
                Ordering::Equal => {
                    return false;
                }
            }
        }

        let raw_node = Box::into_raw(Box::new(AVLNode::new(key, value)));
        self.count += 1;
        let mut new_node = unsafe { raw_node.as_mut().unwrap() };
        let mut new_node_parent = unsafe { parent.as_mut().unwrap() };
        new_node.parent = parent;
        if (is_left) {
            new_node_parent.left = raw_node;
            let smallest = unsafe { self.first.as_ref().unwrap() };
            let ord = new_node.entry.key.cmp(&smallest.entry.key);
            if ord == Ordering::Less {
                self.first = raw_node;
            }
        } else {
            new_node_parent.right = raw_node;
            let largest = unsafe { self.last.as_ref().unwrap() };
            let ord = new_node.entry.key.cmp(&largest.entry.key);
            if ord == Ordering::Greater {
                self.last = raw_node;
            }
        }
        self.rebalance(new_node_parent);

        return true;
    }

    pub fn entry<Q>(&self, search_key: &Q) -> Option<&AVLNode<K, V>>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut target = self.root;
        while let Some(node) = unsafe { target.as_ref() } {
            let ord = search_key.cmp(node.entry.key.borrow());
            match ord {
                Ordering::Less => {
                    target = node.left;
                }
                Ordering::Greater => {
                    target = node.right;
                }
                Ordering::Equal => {
                    return Some(node);
                }
            }
        }
        return None;
    }

    pub fn entry_mut<Q>(&mut self, search_key: &Q) -> Option<&mut AVLNode<K, V>>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut target = self.root;
        while let Some(node) = unsafe { target.as_mut() } {
            let ord = search_key.cmp(node.entry.key.borrow());
            match ord {
                Ordering::Less => {
                    target = node.left;
                }
                Ordering::Greater => {
                    target = node.right;
                }
                Ordering::Equal => {
                    return Some(node);
                }
            }
        }
        return None;
    }

    pub fn contains<Q>(&self, search_key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        return self.entry(search_key).is_some();
    }

    pub fn get<Q>(&self, search_key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        return self.entry(search_key).map(|node| node.value());
    }

    pub fn get_mut<Q>(&mut self, search_key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        return self.entry_mut(search_key).map(|node| node.value_mut());
    }

    pub fn remove<Q>(&mut self, search_key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        let mut target = self.root;
        while let Some(node) = unsafe { target.as_mut() } {
            let ord = search_key.cmp(node.entry.key.borrow());
            match ord {
                Ordering::Less => {
                    target = node.left;
                }
                Ordering::Greater => {
                    target = node.right;
                }
                Ordering::Equal => {
                    let result = self.remove_node(node);
                    return Some(result.value);
                }
            }
        }
        return None;
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        unsafe {
            return Iter {
                head: self.first.as_ref(),
                tail: self.last.as_ref(),
                count: self.len(),
            };
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V> {
        unsafe {
            return IterMut {
                head: self.first.as_mut(),
                tail: self.last.as_mut(),
                count: self.len(),
            };
        }
    }

    pub fn drain(&mut self) -> Drain<K, V> {
        unsafe {
            let d = Drain {
                stack: self.root,
                count: self.len(),
            };
            self.count = 0;
            self.root = ptr::null_mut();
            self.first = ptr::null_mut();
            self.last = ptr::null_mut();

            return d;
        }
    }

    /// Clears the binary tree by re-using parent pointers as a stack instead of DFS recursion. O(n).
    pub fn clear(&mut self) {
        let mut stack: *mut AVLNode<K, V> = self.root;
        while !stack.is_null() {
            //pop the first element off the stack.
            let tmp = stack;
            let pop_node = unsafe { tmp.as_mut().unwrap() };
            stack = pop_node.parent;

            if !pop_node.left.is_null() {
                let left_node = unsafe { pop_node.left.as_mut().unwrap() };
                left_node.parent = stack;
                stack = pop_node.left;
            }

            if !pop_node.right.is_null() {
                let right_node = unsafe { pop_node.right.as_mut().unwrap() };
                right_node.parent = stack;
                stack = pop_node.right;
            }

            let drop_box: Box<AVLNode<K, V>> = unsafe { Box::from_raw(tmp) };
        }
        self.count = 0;
        self.root = ptr::null_mut();
        self.first = ptr::null_mut();
        self.last = ptr::null_mut();
    }
}

impl<K: Ord, V> Drop for AVLTree<K, V> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct Iter<'a, K: Ord, V> {
    head: Option<&'a AVLNode<K, V>>,
    tail: Option<&'a AVLNode<K, V>>,
    count: u32,
}
impl<'a, K: Ord, V> Iterator for Iter<'a, K, V> {
    type Item = &'a Entry<K, V>;
    fn next(&mut self) -> Option<&'a Entry<K, V>> {
        if self.count == 0 {
            return None;
        };
        let node = self.head.unwrap();
        let result = &node.entry;
        self.head = node.successor();
        self.count -= 1;
        return Some(result);
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.count as usize, Some(self.count as usize));
    }
}
impl<'a, K: Ord, V> FusedIterator for Iter<'a, K, V> {}
impl<'a, K: Ord, V> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        return self.count as usize;
    }
}

impl<'a, K: Ord, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a Entry<K, V>> {
        if self.count == 0 {
            return None;
        };
        let node = self.tail.unwrap();
        let result = &node.entry;
        self.tail = node.predecessor();
        self.count -= 1;
        return Some(result);
    }
}

pub struct Drain<K: Ord, V> {
    stack: *mut AVLNode<K, V>,
    count: u32,
}

impl<K: Ord, V> Iterator for Drain<K, V> {
    type Item = Entry<K, V>;
    fn next(&mut self) -> Option<Entry<K, V>> {
        if self.stack.is_null() {
            return None;
        }

        //pop the first element off the stack.
        let tmp = self.stack;
        let pop_node = unsafe { tmp.as_mut().unwrap() };
        self.stack = pop_node.parent;

        if !pop_node.left.is_null() {
            let left_node = unsafe { pop_node.left.as_mut().unwrap() };
            left_node.parent = self.stack;
            self.stack = pop_node.left;
        }

        if !pop_node.right.is_null() {
            let right_node = unsafe { pop_node.right.as_mut().unwrap() };
            right_node.parent = self.stack;
            self.stack = pop_node.right;
        }
        self.count -= 1;
        let drop_box: Box<AVLNode<K, V>> = unsafe { Box::from_raw(tmp) };
        return Some(drop_box.entry);
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.count as usize, Some(self.count as usize));
    }
}

impl<K: Ord, V> Drop for Drain<K, V> {
    fn drop(&mut self) {
        while !self.stack.is_null() {
            //pop the first element off the stack.
            let tmp = self.stack;
            let pop_node = unsafe { tmp.as_mut().unwrap() };
            self.stack = pop_node.parent;

            if !pop_node.left.is_null() {
                let left_node = unsafe { pop_node.left.as_mut().unwrap() };
                left_node.parent = self.stack;
                self.stack = pop_node.left;
            }

            if !pop_node.right.is_null() {
                let right_node = unsafe { pop_node.right.as_mut().unwrap() };
                right_node.parent = self.stack;
                self.stack = pop_node.right;
            }

            let drop_box: Box<AVLNode<K, V>> = unsafe { Box::from_raw(tmp) };
        }
    }
}
impl<K: Ord, V> FusedIterator for Drain<K, V> {}
impl<K: Ord, V> ExactSizeIterator for Drain<K, V> {
    fn len(&self) -> usize {
        return self.count as usize;
    }
}
pub struct IterMut<'a, K: Ord, V> {
    head: Option<&'a mut AVLNode<K, V>>,
    tail: Option<&'a mut AVLNode<K, V>>,
    count: u32,
}
impl<'a, K: Ord, V> Iterator for IterMut<'a, K, V> {
    type Item = &'a mut Entry<K, V>;
    fn next(&mut self) -> Option<&'a mut Entry<K, V>> {
        if self.count == 0 {
            return None;
        };

        self.count -= 1;

        //We have to do this because the borrow checker can't figure out these are separate parts of the struct.
        let head = self.head.take().unwrap() as *mut AVLNode<K, V>;
        let mut result = None;
        unsafe {
            result = Some(&mut (*head).entry);
            self.head = head.as_mut().unwrap().successor_mut();
        }

        return result;
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        return (self.count as usize, Some(self.count as usize));
    }
}
impl<'a, K: Ord, V> FusedIterator for IterMut<'a, K, V> {}
impl<'a, K: Ord, V> ExactSizeIterator for IterMut<'a, K, V> {
    fn len(&self) -> usize {
        return self.count as usize;
    }
}

impl<'a, K: Ord, V> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a mut Entry<K, V>> {
        if self.count == 0 {
            return None;
        };
        self.count -= 1;
        let tail = self.tail.take().unwrap() as *mut AVLNode<K, V>;
        let mut result = None;
        unsafe {
            result = Some(&mut (*tail).entry);
            self.tail = tail.as_mut().unwrap().predecessor_mut();
        }
        return result;
    }
}

#[cfg(test)]
mod test;
