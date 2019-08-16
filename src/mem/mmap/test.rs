#[cfg(test)]
mod test {
    use crate::mem::mmap;
    #[test]
    fn alloc() {
        let result = mmap::alloc_page_aligned(1);
        assert_eq!(mmap::get_page_aligned_size(1), result.size);
        unsafe{
        mmap::free_page_aligned(result.memory, result.size);
    }
    }

}
// struct MyStruct {
 
// }
// struct blah{
//     q : IndexSpinlock<MyStruct>,
// }