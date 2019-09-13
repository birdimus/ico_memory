use crate::mem::QueueUsize;
// use crate::mem::queue::Swap;
use crate::sync::IndexSpinlock;
use core::num::NonZeroUsize;
use core::sync::atomic::AtomicUsize;
use std::thread;
use std::time::Instant;

static mut BUFFER: [usize; 4096] = [0; 4096];
static mut QUEUE_PTR : *mut AtomicUsize = unsafe{&BUFFER[0] as *const usize as *mut AtomicUsize};
// unsafe { Swap::<[usize; 4096], [AtomicUsize; 4096]>::get([0; 4096]) };
static QUEUE: QueueUsize =
    unsafe { QueueUsize::from_static(&QUEUE_PTR, 4096) };

static LOCK: IndexSpinlock = IndexSpinlock::new(0);
#[test]
fn mpmc() {
    // assert_eq!(core::mem::size_of::<Option<NonZeroUsize>>(), core::mem::size_of::<usize>());
    let _t = LOCK.lock();
    for i in 0..4096 {
        QUEUE.enqueue(NonZeroUsize::new(i + 1).unwrap());
    }
    QUEUE.clear();
}

#[test]
fn mpmc_local() {
    unsafe {
        let mut buffer_local: [usize; 4096] = [0; 4096];
        let mut buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
        let queue_local =
            QueueUsize::from_static(&buffer_ptr, 4096);

        for i in 0..4096 {
            queue_local.enqueue(NonZeroUsize::new(i + 1).unwrap());
        }
    }
}

#[test]
fn mpmc_dequeue() {
    unsafe {
        let mut buffer_local: [usize; 4096] = [0; 4096];
        let mut buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
        let m =
            QueueUsize::from_static(&buffer_ptr, 4096);
        for _j in 0..20 {
            for i in 0..4096 {
                assert_eq!(m.enqueue(NonZeroUsize::new(i + 1).unwrap()), true);
            }
            for i in 0..4096 {
                assert_eq!(m.dequeue().unwrap().get(), i + 1);
            }
        }
    }
}

#[test]
fn mpmc_enqueue_dequeue() {
    unsafe {
        let mut buffer_local: [usize; 4096] = [0; 4096];
        let mut buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
        let m =
            QueueUsize::from_static(&buffer_ptr, 4096);
        for _j in 0..20 {
            for i in 0..4096 {
                assert_eq!(m.enqueue(NonZeroUsize::new(i + 1).unwrap()), true);
                assert_eq!(m.dequeue().unwrap().get(), i + 1);
            }
        }
    }
}

#[test]
fn mpmc_enqueue2_dequeue_wrap() {
    unsafe {
        let mut buffer_local: [usize; 4096] = [0; 4096];
        let mut buffer_ptr = &mut buffer_local[0] as *mut usize as *mut AtomicUsize;
        // unsafe { Swap::<[usize; 4096], [AtomicUsize; 4096]>::get([0; 4096]) };
        let m = QueueUsize::from_static(&buffer_ptr, 4096);
        for _j in 0..20 {
            for i in 0..4096 {
                m.enqueue(NonZeroUsize::new(i + 1).unwrap());
                m.enqueue(NonZeroUsize::new(i + 1).unwrap());
                m.dequeue();
                //assert_eq!(m.dequeue().unwrap(), i/2);
            }
        }
    }
}

// // /*
// #[test]
// fn mpmc_enqueue_dequeue2_wrap() {
//     let m = Arc::new(Queue::<usize>::new(4096));
//     for j in 0..20{
//         for i in 0..4096{

//             m.enqueue(i);
//             assert_eq!(m.dequeue().unwrap(), i);
//             assert_eq!(m.dequeue(), None);
//         }

//     }

// }
// // */
#[test]
fn mpmc_threads() {
    // unsafe{
    let _t = LOCK.lock();
    let now = Instant::now();

    //let t =  &QUEUE;//Arc::new(QUEUE);
    for i in 0..4096 {
        QUEUE.enqueue(NonZeroUsize::new(i + 1).unwrap());
    }
    let mut children = vec![];

    for _i in 0..4 {
        // let mut t = m.clone();
        // Spin up another thread
        children.push(thread::spawn(|| {
            let mut data = Vec::with_capacity(1024);
            for _j in 0..256 {
                let mut x = 0;
                for _k in 0..1024 {
                    let q = QUEUE.dequeue();
                    // while(q.is_none()){
                    //     std::thread::yield_now();
                    //     q = t.dequeue();
                    // }
                    if q.is_some() {
                        data.push(q.unwrap());
                        x += 1;
                    }
                    assert_eq!(q.is_some(), true);
                }
                assert_eq!(x, 1024);
                let mut y = 0;
                for _k in 0..1024 {
                    let val = data.pop().unwrap();
                    //if(q.is_some()){
                    //let val = q.unwrap();
                    let r = QUEUE.enqueue(val);
                    // let mut kk = 0;
                    //while(r == false){
                    // kk += 1;
                    //    std::thread::yield_now();
                    //     r = t.enqueue(val);
                    // }
                    //  if(kk > 0){
                    //  println!("overage {}", kk);
                    //  }
                    assert_eq!(r, true);
                    y += 1;
                    // }
                }
                assert_eq!(x, y);

                // Loop unrolling when it shouldn't.
                // std::thread::yield_now();
            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    let mut ints: Vec<u32> = Vec::with_capacity(4096);
    for _i in 0..4096 {
        ints.push(0);
    }
    for _i in 0..4096 {
        //if m.dequeue().is_some(){
        let r = QUEUE.dequeue();
        if r.is_some() {
            ints[r.unwrap().get() - 1 as usize] += 1;
        }

        //}
        //assert_eq!(m.dequeue().unwrap(),i);
        //assert_eq!(m.dequeue().is_some(),true);
    }
    for _i in 0..4096 {
        assert_eq!(QUEUE.dequeue(), None);
    }
    let mut count = 0;
    for i in 0..4096 {
        if ints[i] == 1 {
            count += 1;
        }
        //assert_eq!(ints[i],true);
    }
    assert_eq!(count, 4096);
    println!("count {}", count);
    println!("{}", now.elapsed().as_micros());
    QUEUE.clear();
    // }
}
#[test]
fn mpmc_threads2() {
    // unsafe{
    let _t = LOCK.lock();
    let now = Instant::now();

    for i in 0..4096 {
        QUEUE.enqueue(NonZeroUsize::new(i + 1).unwrap());
    }
    let mut children = vec![];

    for i in 0..4 {
        // Spin up another thread
        children.push(thread::spawn(move || {
            for _j in 0..1024 {
                for _k in 0..4096 {
                    QUEUE.dequeue();
                }

                for _k in 0..4096 {
                    //assert_eq!(t.enqueue(data.pop().unwrap()), true);
                    QUEUE.enqueue(NonZeroUsize::new(i + 1).unwrap());
                }
            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    println!("{}", now.elapsed().as_micros());
    QUEUE.clear();
    // }
}
