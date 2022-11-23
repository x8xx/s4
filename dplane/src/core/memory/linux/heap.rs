use std::mem::size_of;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use crate::core::memory::array::Array;


pub struct Heap {
    data: *mut u8,
    size: usize,
    next_pos: isize,
}


impl Heap {
    pub fn new(size: usize) -> Self {
        let data = if size != 0 {
            unsafe {
                libc::malloc(size * size_of::<u8>()) as *mut u8 
            }
        } else {
            null_mut() as *mut u8
        };

        Heap {
            data,
            size,
            next_pos: 0,
        }
    }


    pub fn malloc<T>(&mut self, size: usize) -> Array<T> {
        let start_pos = self.next_pos;
        let end_pos = size_of::<T>() * size;
        self.next_pos= end_pos as isize + 1;

        unsafe {
            Array::new_manual(self.data.offset(start_pos) as *mut T, size)
        }
    }


    pub fn free(&self) {
        unsafe {
            libc::free(self.data as *mut c_void)
        }
    }
}
