#[cfg(test)]
mod test {

    use crate::mem::IndexedData;
    use crate::mem::IndexedDataStore;
    use crate::mem::IndexedHandle;
    use crate::mem::IndexedRef;
    use core::mem::MaybeUninit;
    static mut _BUFFER: [u8; 1024 * core::mem::size_of::<IndexedData<u32>>()] =
        [0; 1024 * core::mem::size_of::<IndexedData<u32>>()];
    static mut _BUFFER_PTR: *mut MaybeUninit<IndexedData<u32>> =
        unsafe { &_BUFFER[0] as *const u8 as *mut u8 as *mut MaybeUninit<IndexedData<u32>> };

    // static mut buf_ref : &mut MaybeUninit< &[IndexedData<Vec<u32>>]>= &mut BUF2;

    static mut _IDS: IndexedDataStore<u32> =
        unsafe { IndexedDataStore::from_raw(&_BUFFER_PTR, 1024) };


    #[test]
    fn size() {
        assert_eq!(core::mem::size_of::<Option<IndexedRef<Vec<u32>>>>(), core::mem::size_of::<IndexedRef<Vec<u32>>>());
    }
    #[test]
    fn lifetime() {
        let hr: IndexedRef<Vec<u32>>;
        let _rr: &Vec<u32>;
        {
            let mut buffer2: [MaybeUninit<IndexedData<Vec<u32>>>; 1024] =
                unsafe { MaybeUninit::uninit().assume_init() };
            let ptr: *mut MaybeUninit<IndexedData<Vec<u32>>> = buffer2.as_mut_ptr();

            {
                let ids: IndexedDataStore<Vec<u32>> =
                    unsafe { IndexedDataStore::from_raw(&ptr, 1024) };
                let mut v = Vec::<u32>::with_capacity(1);
                v.push(9);
                let tmp = ids.store(v).unwrap();
                hr = ids.retain(tmp).unwrap();
                _rr = unsafe { ids.get(&hr) };
            }
        }
        //NOTE - this should be a compiler error, and is!
        // let hr2 : HandleRef<Vec<u32>> = hr;
        // rr.push(0);
    }
    #[test]
    fn insert_remove() {
        {
            let mut buffer2: [MaybeUninit<IndexedData<Vec<u32>>>; 1024] =
                unsafe { MaybeUninit::uninit().assume_init() };
            let ptr: *mut MaybeUninit<IndexedData<Vec<u32>>> = buffer2.as_mut_ptr();

            {
                let ids: IndexedDataStore<Vec<u32>> =
                    unsafe { IndexedDataStore::from_raw(&ptr, 1024) };

                let mut handles = Vec::<IndexedHandle>::with_capacity(1024);
                for k in 0..16 {
                    for i in 0..1024 {
                        let mut v = Vec::<u32>::with_capacity(1);
                        v.push(i);
                        let tmp = ids.store(v).unwrap();
                        handles.push(tmp);
                        if k == 0 {
                            assert_eq!(ids.high_water_mark(), i + 1);
                        } else {
                            assert_eq!(ids.high_water_mark(), 1024);
                        }
                        assert_eq!(ids.active(), i + 1);
                        assert_eq!(ids.capacity(), 1024);
                    }

                    for i in 0..1024 {
                        let q = ids.retain(handles[i]).unwrap();
                        assert_eq!(unsafe { ids.get(&q)[0] }, i as u32);
                        ids.release(q);
                    }
                    for i in 0..1024 {
                        ids.free(handles[i]);

                        assert_eq!(ids.high_water_mark(), 1024);
                        assert_eq!(ids.active(), 1024 - (i as u32) - 1);
                        assert_eq!(ids.capacity(), 1024);
                    }
                    // for i in 0..1024{
                    // {
                    // let r = ids.get(handles[i]);
                    // assert_eq!(r, None );
                    // }
                    // }
                    handles.clear();
                }
            }
        }
    }

    #[test]
    fn safety() {
        let mut buffer2: [MaybeUninit<IndexedData<Vec<u32>>>; 1024] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let ptr: *mut MaybeUninit<IndexedData<Vec<u32>>> = buffer2.as_mut_ptr();
        let ids = unsafe { IndexedDataStore::from_raw(&ptr, 1024) };

        let mut handles = Vec::<IndexedHandle>::with_capacity(1024);
        for k in 0..16 {
            for i in 0..1024 {
                let mut v = Vec::<u32>::with_capacity(1);
                v.push(i);
                handles.push(ids.store(v).unwrap());
            }

            for i in 0..1024 {
                for j in 0..k + 1 {
                    if j == k {
                        let q = ids.retain(handles[i + 1024 * j]).unwrap();
                        assert_eq!(unsafe { ids.get(&q)[0] }, i as u32);
                        ids.release(q);
                    } else {
                        assert_eq!(ids.retain(handles[i + 1024 * j]), None);
                    }
                }
            }
            for i in 0..1024 {
                for j in 0..k + 1 {
                    if j == k {
                        assert_eq!(ids.free(handles[i + 1024 * j]), true);
                    } else {
                        //ids.free(handles[i + 1024*j]);
                        assert_eq!(ids.free(handles[i + 1024 * j]), false);
                    }
                }
            }
        }
    }
}
