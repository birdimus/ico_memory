#![cfg_attr(not(any(test, feature = "use-std")), no_std)]

pub mod sync;

#[cfg(not(any(test, feature = "use-std")))]
#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
