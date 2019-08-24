#[cfg(test)]
mod test {


	use crate::collections::avl;

	#[test]
    fn init() {
    	let tree = avl::AVLTree::<u8,i32>::new();

    }

    #[test]
    fn insert() {
    	let mut tree = avl::AVLTree::<u8,i32>::new();
    	for i in 0..=255{
    		assert_eq!(tree.count(), i);
    		assert_eq!(true, tree.insert(i as u8, -(i as i32)));
    		assert_eq!(tree.count(), i+1);
    	}
    	for i in 0..=255{

    		assert_eq!(false, tree.insert(i as u8, -(i as i32)), "{ }", i);
    		assert_eq!(tree.count(), 256);
    	}

    	for i in 0..=255{
    		assert_eq!(true, tree.contains(&i));
    	}

    	for i in 0..=255{
    		let tmp = tree.entry(&i).unwrap();
    		assert_eq!(*tmp.value(), -(i as i32));
    		assert_eq!(true, tmp.balance() > -2 && tmp.balance() < 2);

    		assert_eq!(*tree.get(&i).unwrap(), -(i as i32));
    	}

    }

    #[test]
    fn remove() {
    	let mut tree = avl::AVLTree::<u8,i32>::new();
    	for i in 0..=255{
    		assert_eq!(tree.count(), i);
    		assert_eq!(true, tree.insert(i as u8, -(i as i32)));
    		assert_eq!(tree.count(), i+1);
    	}

    	// for i in 0..=255{
    	// 	// assert_eq!(tree.count(), i);
    	// 	assert_eq!(-(i as i32), tree.remove(&(i as u8)).unwrap());
    	// 	// assert_eq!(tree.count(), i+1);
    	// }

    }



}