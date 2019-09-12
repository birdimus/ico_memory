#[cfg(test)]
mod test {

    use crate::collections::indexed_data_store::IndexedDataStore;
    use crate::collections::indexed_data_store::Handle;
    // use std::time::Instant;

    #[test]
    fn insert_remove() {
        let mut idx = IndexedDataStore::<Vec<u32>>::with_capacity(1024);
        let mut handles = Vec::<Handle>::with_capacity(1024);
        for _k in 0..16{
            for i in 0..1024{

                let mut v = Vec::<u32>::with_capacity(1);
                v.push(i);
                handles.push(idx.retain(v));
                assert_eq!(idx.high_water_mark(), i+1);
                assert_eq!(idx.len(), i+1);
                assert_eq!(idx.capacity(), 1024);
            }
            
            for i in 0..1024{
                assert_eq!(idx.get(handles[i]).unwrap()[0], i as u32 );
            }
            for i in 0..1024{
                assert_eq!(idx.release(handles[i]).unwrap()[0], i as u32 );
                assert_eq!(idx.high_water_mark(), 1024);
                assert_eq!(idx.len(), 1024-(i as u32)-1);
                assert_eq!(idx.capacity(), 1024);
            }
            for i in 0..1024{
                assert_eq!(idx.get(handles[i]), None );
            }
            handles.clear();
            idx.clear();
            assert_eq!(idx.high_water_mark(), 0);
            assert_eq!(idx.len(), 0);
            assert_eq!(idx.capacity(), 1024);
        }
    }

    #[test]
    fn safety() {
        let mut idx = IndexedDataStore::<Vec<u32>>::with_capacity(1024);
        let mut handles = Vec::<Handle>::with_capacity(1024);
        for k in 0..16{
            for i in 0..1024{

                let mut v = Vec::<u32>::with_capacity(1);
                v.push(i);
                handles.push(idx.retain(v));
            }
            
            for i in 0..1024{
                for j in 0..k{
                    if j==k-1{
                        assert_eq!(idx.get(handles[i + 1024*j]).unwrap()[0], i as u32 );
                    }
                    else{
                         assert_eq!(idx.get(handles[i + 1024*j]), None);
                    }
                    
                }
            }
            for i in 0..1024{
                 for j in 0..k{
                    if j==k-1{
                        assert_eq!(idx.release(handles[i + 1024*j]).unwrap()[0], i as u32 );
                    }
                    else{
                        assert_eq!(idx.release(handles[i + 1024*j]), None );
                    }
                    
                }
                

            }
            
        }
    }

    #[test]
    fn iter() {
        let mut idx = IndexedDataStore::<Vec<u32>>::with_capacity(1024);
        let mut handles = Vec::<Handle>::with_capacity(1024);
        for i in 0..1024{

            let mut v = Vec::<u32>::with_capacity(1);
            v.push(i);
            handles.push(idx.retain(v));
        }
        //release all the evens.
        for i in 0..512{
            assert_eq!(idx.release(handles[2*i]).unwrap()[0], 2*i as u32 );
        }

        let mut iter = idx.iter();
        for i in 0..iter.len(){
             // println!("iter {}", i);
             assert_eq!( (iter.next().unwrap()[0]), 2*i as u32 + 1);
        }
        assert_eq!( iter.next(), None);
    }
    #[test]
    fn iter_mut() {
        let mut idx = IndexedDataStore::<Vec<u32>>::with_capacity(1024);
        let mut handles = Vec::<Handle>::with_capacity(1024);
        for i in 0..1024{

            let mut v = Vec::<u32>::with_capacity(1);
            v.push(i);
            handles.push(idx.retain(v));
        }
        //release all the evens.
        for i in 0..512{
            assert_eq!(idx.release(handles[2*i]).unwrap()[0], 2*i as u32 );
        }

        let mut iter = idx.iter_mut();
        for i in 0..iter.len(){
             // println!("iter {}", i);
             let q = iter.next().unwrap();

             assert_eq!( (q[0]), 2*i as u32 + 1);
        }
        assert_eq!( iter.next(), None);
    }

}
