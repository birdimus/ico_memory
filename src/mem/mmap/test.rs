#[cfg(test)]
mod test {
    use crate::mem::mmap;
    #[test]
    fn alloc() {
        let size = mmap::get_page_aligned_size(1);
        let result = mmap::alloc_page_aligned(size);
        unsafe {
            mmap::free_page_aligned(result.memory, result.size);
        }
    }
}
// struct MyStruct {

// }
// struct blah{
//     q : IndexSpinlock<MyStruct>,
// }
