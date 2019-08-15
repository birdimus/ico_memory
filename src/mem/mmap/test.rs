#[cfg(test)]
mod test {
    use crate::mem::mmap;
    #[test]
    fn alloc() {
        let result = mmap::alloc_page_aligned(1);
        assert_eq!(mmap::get_page_aligned_size(1), result.size);
        mmap::free_page_aligned(result);
    }

}
