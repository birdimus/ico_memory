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
    		assert_eq!(tree.len(), i);
    		assert_eq!(true, tree.insert(i as u8, -(i as i32)));
    		assert_eq!(tree.len(), i+1);
    	}
    	for i in 0..=255{

    		assert_eq!(false, tree.insert(i as u8, -(i as i32)), "{ }", i);
    		assert_eq!(tree.len(), 256);
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
    		assert_eq!(tree.len(), i);
    		assert_eq!(true, tree.insert(i as u8, -(i as i32)));
    		assert_eq!(tree.len(), i+1);
    	}

    	for i in 0..=255{

    		//Check validity as we remove things
    		for j in i..=255{
	    		let tmp = tree.entry(&j).unwrap();
	    		assert_eq!(*tmp.value(), -(j as i32));
	    		assert_eq!(true, tmp.balance() > -2 && tmp.balance() < 2);

	    		assert_eq!(*tree.get(&j).unwrap(), -(j as i32));
	    	}



    		let v = tree.remove(&(i as u8));

    		//Make sure it's gone.
    		assert_eq!(tree.get(&(i as u8)), None);

    		// Make sure it returned something, and the something is the right thing.
    		assert_eq!(v.is_some(), true);
    		assert_eq!(-(i as i32), v.unwrap());

    		// Check the count.
    		assert_eq!(tree.len(), 255 - i as u32);

    	}

    }
    #[test]
    fn iterator() {
    	let mut tree = avl::AVLTree::<u8,i32>::new();
    	for i in 0..=255{
    		assert_eq!(tree.len(), i);
    		assert_eq!(true, tree.insert(i as u8, -(i as i32)));
    		assert_eq!(tree.len(), i+1);
    	}

    	let mut iter = tree.iter();
    	let mut blah = iter.next();
    	while blah.is_some(){
    		
    		blah = iter.next();
    	}

    }


}