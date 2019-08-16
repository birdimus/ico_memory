#[cfg(test)]
mod test {
	const SIZE : u32 = 64;
    use crate::mem::memory_pool::MemoryPool;
    #[test]
    fn alloc() {
        let mp = MemoryPool::<64, 2048, 1024>::new();
        for i in 0..2048 {
            for j in 0..1024 {
                mp.allocate();
            }
        }
    }

}
