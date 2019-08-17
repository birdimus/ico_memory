#[cfg(test)]
mod test {
    use crate::sync::index_lock::IndexSpinlock;
    use std::thread;
    use std::sync::Arc;
    use std::time::Instant;




    #[test]
    fn indexspinlock_writers() {
        let now = Instant::now();

        let mut spinlock = IndexSpinlock::new(0);
        spinlock.set(1);
        let index_lock : Arc<IndexSpinlock> = Arc::new(spinlock);
        let thread_count = 16;
        {
            
            let mut children = vec![];
            
            
            for i in 0..thread_count {
                let p = index_lock.clone();
                // Spin up another thread
                children.push(thread::spawn(move || {
                    let mut ve = p.lock();
                    for i in 0..101{

                        ve.write(ve.read() + i);
                    }
                }));
            }

            for child in children {
                // Wait for the thread to finish. Returns a result.
                let _ = child.join();
            }
        }
        spinlock = Arc::try_unwrap(index_lock).ok().unwrap();
        //let ve = index_lock.lock();
        assert_eq!(spinlock.get(), 1 + 5050* thread_count);

        println!("spinlock writers {}", now.elapsed().as_micros());
    }
}