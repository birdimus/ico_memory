#[cfg(test)]
mod test {

    use crate::collections::binary_heap::BinaryHeap;
    use crate::collections::binary_heap::Handle;
    use std::time::Instant;

    #[test]
    fn insert() {
        let mut b: BinaryHeap<i32> = BinaryHeap::new();

        for i in 0..100 {
            let h = b.push(i);
            assert_eq!(b.get(h).unwrap(), &i);
        }

        assert_eq!(b.heap[0], 0xFFFFFFFF);
        for i in 0..100 {
            unsafe {
                assert_eq!(b.data[i].assume_init(), i as i32);
            }
            assert_eq!(b.heap[1 + i], i as u32);
        }
    }

    #[test]
    fn remove() {
        let mut b: BinaryHeap<i32> = BinaryHeap::new();

        for i in 0..100 {
            b.push(i);
        }

        for i in 0..100 {
            assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
        }
    }
    #[test]
    fn insert_index() {
        let mut b: BinaryHeap<i32> = BinaryHeap::new();
        for i in 0..100 {
            b.push(i);
            b.push(99 - i);
        }

        for i in 0..200 {
            unsafe {
                assert_eq!(b.heap_index[b.heap[i + 1] as usize].index, i as u32 + 1);
                // assert_eq!(b.heap[b.heap_index[i] as usize], i as u32);
            }
            // assert_eq!(b.heap[1+i], i as u32);
        }
    }

    #[test]
    fn complex_insert_remove() {
        let mut b: BinaryHeap<i32> = BinaryHeap::new();
        for j in 0..100 {
            for i in 0..100 {
                let h1 = b.push(i);

                assert_eq!(b.get(h1).unwrap(), &i);
                let t = 99 - i;
                let h2 = b.push(99 - i);
                assert_eq!(b.get(h2).unwrap(), &t);
            }
            for i in 0..200 {
                unsafe {
                    assert_eq!(b.heap[b.heap_index[i].index as usize], i as u32);
                }
            }
            for i in 0..100 {
                assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
                assert_eq!(b.pop().unwrap(), i as i32, "binary heap pop {}", i);
                let count = 99 - i;
                for k in 0..(2 * count) {
                    unsafe {
                        assert_eq!(b.heap_index[b.heap[k + 1] as usize].index, k as u32 + 1);
                    }
                }
            }
        }
    }

    #[test]
    fn replace() {
        let mut b: BinaryHeap<i32> = BinaryHeap::new();

        let mut v : Vec<Handle> = Vec::with_capacity(30);
        for i in 0..30 {
            v.push (b.push(i));
        }

        // This resizes keys, some larger some smaller.
        for i in 0..30 {
            b.replace(v[i], (i as i32%3)*20  );
        }

        for j in 0..3 {
            for i in 0..10 {
                    assert_eq!(b.pop().unwrap(), 20*j as i32, "binary heap pop {}", j);
                
            }
        }

    }
}
