use std::ops::Index;
use std::ops::IndexMut;
use std::marker::Send;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;
use std::ffi::c_void;


#[derive(Clone)]
pub struct Array<T> {
    data: *mut T,
    len: usize,
}

unsafe impl<T> Send for Array<T> {}
unsafe impl<T> Sync for Array<T> {}

impl<T> Array<T> {
    pub fn new(len: usize) -> Self {
        let data = if len != 0 {
            unsafe {
                libc::malloc(len * size_of::<T>()) as *mut T
            }
        } else {
            null_mut() as *mut T
        };

        Array {
            data,
            len,
        }
    }

    pub fn new_manual(data: *mut T, len: usize) -> Self {
        Array {
            data,
            len,
        }
    }

    pub fn init(&mut self, index: usize,  value: T) {
        unsafe {
            std::ptr::write::<T>(self.data.offset(index as isize), value);
            // *self.data.offset(index as isize) = value; 
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.data
    }


    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &mut [T] {
        unsafe {
            from_raw_parts_mut::<T>(self.data, self.len)
        }

    }

    pub fn get(&self, index: usize) -> &mut T {
        unsafe {
            &mut *self.data.offset(index as isize)
        }
    }

    pub fn free(self) {
        unsafe {
            libc::free(self.data as *mut c_void)
        }
    }

    pub fn clone(&self) -> Self {
        Array {
            data: self.data,
            len: self.len,
        }
    }
}


impl<U: Copy> Array<U> {
    pub fn deepcopy(&self, dst: &mut Array<U>) {
        for i in 0..self.len() {
            dst[i] = self[i];
        }
    }
}


impl<T> Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, i: usize) ->  &Self::Output {
        unsafe {
            &*self.data.offset(i as isize)
        }
    }
}

impl<T> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        unsafe {
            // &mut *self.data.offset(i as isize) as &mut T
            &mut *self.data.offset(i as isize)
        }
    }
}
