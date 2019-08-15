#[cfg(test)]
mod test {
    use crate::mem::memory_pool64::MemoryPool64;
    #[test]
    fn alloc() {
        let mp = MemoryPool64::new();
        for i in 0..2048 {
            for j in 0..1024 {
                mp.allocate();
            }
        }
    }

}
