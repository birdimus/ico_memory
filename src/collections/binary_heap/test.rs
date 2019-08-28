#[cfg(test)]
mod test {


	use crate::collections::binary_heap::BinaryHeap;
	use std::time::Instant;

	#[test]
    fn insert() {
        let mut b : BinaryHeap<i32> = BinaryHeap::new();

        for i in 0..100{
            b.push(i);
        }

        assert_eq!(b.heap[0], 0xFFFFFFFF);
        for i in 0..100{
            unsafe{
                assert_eq!(b.data[i].assume_init(), i as i32);
            }   
            assert_eq!(b.heap[1+i], i as u32);
        }
    }

    #[test]
    fn remove() {
        let mut b : BinaryHeap<i32> = BinaryHeap::new();

        for i in 0..100{
            b.push(i);
        }

        for i in 0..100{
            assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
        }
    }
    #[test]
    fn insert_index() {
        let mut b : BinaryHeap<i32> = BinaryHeap::new();
        for i in 0..100{
            b.push(i);
            b.push(99 - i);
        }

        for i in 0..200{
            unsafe{
                assert_eq!(b.heap[b.heap_index[i] as usize], i as u32);
            }   
            // assert_eq!(b.heap[1+i], i as u32);
        }

    }

    #[test]
    fn complex_insert_remove() {
        let mut b : BinaryHeap<i32> = BinaryHeap::new();
        for j in 0..100{
            for i in 0..100{
                b.push(i);
                b.push(99 - i);
            }
            for i in 0..200{
                unsafe{
                    assert_eq!(b.heap[b.heap_index[i] as usize], i as u32);
                }   
            }
            for i in 0..100{
                assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
                assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
                let count = 99-i;
                for k in 0..(2*count){
                    unsafe{
                        assert_eq!(b.heap[b.heap_index[k] as usize], k as u32);
                    }   
                }
            }
        }
    
    }

}