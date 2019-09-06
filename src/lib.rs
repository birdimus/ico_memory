#![cfg_attr(not(any(test, feature = "std")), no_std)]

pub mod collections;
pub mod mem;
pub mod sync;

#[cfg(not(any(test, feature = "std")))]
#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
