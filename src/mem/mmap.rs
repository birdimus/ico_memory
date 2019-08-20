extern crate libc;
use core::ptr;

#[derive(Copy, Clone, Debug)]
#[repr(C, align(16))]
pub(crate) struct MapAlloc {
    pub(crate) memory: *mut u8,
    pub(crate) size: usize,
}

impl MapAlloc {
    #[inline(always)]
    pub const fn null() -> MapAlloc {
        return MapAlloc {
            memory: ptr::null_mut(),
            size: 0,
        };
    }
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        return self.size == 0;
    }
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, offset: isize) -> *mut u8 {
        return self.memory.offset(offset);
    }
}
/// Return the system page size.
#[inline(always)]
pub fn page_size() -> usize {
    unsafe {
        return libc::sysconf(libc::_SC_PAGESIZE) as usize;
    }
}

#[inline(always)]
pub fn get_page_aligned_size(size: usize) -> usize {
    let page_size = page_size(); //cache this?
    let page_size_mask = page_size - 1;

    if (size & page_size_mask) == 0 {
        return size;
    }

    return page_size + (size & !page_size_mask);
}

#[inline(always)]
pub(crate) unsafe fn free_page_aligned(ptr: *mut u8, size: usize) {
    libc::munmap(ptr as *mut libc::c_void, size);
}

#[inline(always)]
pub(crate) fn alloc_page_aligned(alloc_size: usize) -> MapAlloc {
    // let alloc_size = get_page_aligned_size(size);
    unsafe {
        let p: *mut libc::c_void = libc::mmap(
            core::ptr::null_mut(),
            alloc_size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
            -1, //no file
            0,
        ); //no offset

        if p == libc::MAP_FAILED {
            return MapAlloc {
                size: 0,
                memory: core::ptr::null_mut(),
            };
        }
        return MapAlloc {
            size: alloc_size,
            memory: p as *mut u8,
        };
    }
}

#[cfg(test)]
mod test;
