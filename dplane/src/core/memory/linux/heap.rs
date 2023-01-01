use std::mem::size_of;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::core::memory::vector::VectorMeta;


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
        let end_pos = start_pos + (size_of::<T>() * size) as isize;
        self.next_pos= end_pos as isize + 1;

        unsafe {
            Array::new_manual(self.data.offset(start_pos) as *mut T, size)
        }
    }


    pub fn vec_malloc<T: Copy>(&mut self, size: usize, extend_size: usize) -> Vector<T> {
        // data
        let data_pos = self.next_pos;
        let end_pos = data_pos + (size_of::<T>() * size) as isize;
        self.next_pos= end_pos as isize + 1;

        // meta
        let meta_pos = self.next_pos;
        let end_pos = meta_pos + size_of::<VectorMeta>() as isize;
        self.next_pos= end_pos as isize + 1;

        unsafe {
            let meta = self.data.offset(meta_pos) as *mut VectorMeta;
            (*meta).pos = 0;
            (*meta).len = size;
            (*meta).extend_size = extend_size;
            Vector::new_manual(self.data.offset(data_pos) as *mut T, meta)
        }
    }


    pub fn free(&self) {
        unsafe {
            libc::free(self.data as *mut c_void)
        }
    }
}
