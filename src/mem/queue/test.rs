use super::*;
use crate::mem::queue::Queue;
use std::thread;
use std::sync::Arc;
use std::time::Instant;
use core::num::NonZeroUsize;




#[test]
fn mpmc() {
	// assert_eq!(core::mem::size_of::<Option<NonZeroUsize>>(), core::mem::size_of::<usize>());
    let  m = Queue::new(4096);
     for i in 0..4096{
         m.enqueue(NonZeroUsize::new(i+1).unwrap());
     }

}
#[test]
fn mpmc_dequeue() {
    let  m = Queue::new(4096);
    for j in 0..20{

	     for i in 0..4096{
	         assert_eq!(m.enqueue(NonZeroUsize::new(i+1).unwrap()), true);
	     }
	     for i in 0..4096{
	         assert_eq!(m.dequeue().unwrap().get(), i+1);
	     }
       
    }
}


#[test]
fn mpmc_enqueue_dequeue() {
    let  m = Queue::new(4096);
    for j in 0..20{
        for i in 0..4096{
            assert_eq!(m.enqueue(NonZeroUsize::new(i+1).unwrap()), true);
            assert_eq!(m.dequeue().unwrap().get(), i+1);
        }
        
    }
}

#[test]
fn mpmc_enqueue2_dequeue_wrap() {
    let  m = Queue::new(4096);
    for j in 0..20{
        for i in 0..4096{
            m.enqueue(NonZeroUsize::new(i+1).unwrap());
            m.enqueue(NonZeroUsize::new(i+1).unwrap());
            m.dequeue();
            //assert_eq!(m.dequeue().unwrap(), i/2);
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
    let now = Instant::now();
    let m = Arc::new(Queue::new(4096));
    for i in 0..4096 {
        m.enqueue(NonZeroUsize::new(i+1).unwrap());
    }
    let mut children = vec![];
    
    for i in 0..4 {
        let mut t = m.clone();
        // Spin up another thread
        children.push(thread::spawn(move || {
            let mut data = Vec::with_capacity(1024);
            for j in 0..256 {
                
                let mut x = 0;
                for k in 0..1024 {
                    let mut q = t.dequeue();
                   // while(q.is_none()){
                   //     std::thread::yield_now();
                   //     q = t.dequeue();
                   // }
                    if(q.is_some()){
                        data.push(q.unwrap());
                        x+=1;
                    }
                    assert_eq!(q.is_some(), true);
                    
                }
                assert_eq!(x,1024);
                let mut y = 0;
                for k in 0..1024 {
                    let val = data.pop().unwrap();
                    //if(q.is_some()){
                        //let val = q.unwrap();
                        let mut r = t.enqueue(val);
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
                        y+=1;
                   // }
                    
                }
                assert_eq!(x,y);
                
                // Loop unrolling when it shouldn't.
                // std::thread::yield_now();
            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }


    let mut ints : Vec<u32> = Vec::with_capacity(4096);
    for i in 0..4096 {
        ints.push(0);
    }
    for i in 0..4096 {
        //if m.dequeue().is_some(){
            let r = m.dequeue();
            if(r.is_some()){
                ints[r.unwrap().get()-1 as usize] +=1;
            }
            

        //}
        //assert_eq!(m.dequeue().unwrap(),i);
        //assert_eq!(m.dequeue().is_some(),true);
    }
    for i in 0..4096 {
        assert_eq!(m.dequeue(), None);
    }
    let mut count = 0;
    for i in 0..4096 {
        if ints[i] == 1{        count+=1; }
        //assert_eq!(ints[i],true);
    }
    assert_eq!(count,4096);
    println!("count {}", count);
    println!("{}", now.elapsed().as_micros());
}
#[test]
fn mpmc_threads2() {
    let now = Instant::now();
    let m = Arc::new(Queue::new(4096));
    for i in 0..4096 {
        m.enqueue(NonZeroUsize::new(i+1).unwrap());
    }
    let mut children = vec![];
    
    for i in 0..4 {
        let mut t = m.clone();
        // Spin up another thread
        children.push(thread::spawn(move || {
            for j in 0..1024 {
                for k in 0..4096 {
                    t.dequeue();
                    
                }

                for k in 0..4096 {
                    //assert_eq!(t.enqueue(data.pop().unwrap()), true);
                    t.enqueue(NonZeroUsize::new(i+1).unwrap());
                }

            }
        }));
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }


    println!("{}", now.elapsed().as_micros());
}