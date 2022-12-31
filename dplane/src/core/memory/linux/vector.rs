use std::ops::Index;
use std::ops::IndexMut;
use std::marker::Send;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;
use std::os::raw::c_char;
use std::ffi::c_void;


#[derive(Clone, Copy)]
pub struct Vector<T> {
    data: *mut T,
    meta: *mut VectorMeta,
}

struct VectorMeta {
    pos: usize,
    len: usize,
    extend_size: usize, 
}

unsafe impl<T> Send for Vector<T> {}
unsafe impl<T> Sync for Vector<T> {}

impl<T: Copy> Vector<T> {
    pub fn new(len: usize, extend_size: usize) -> Self {
        let data = if len != 0 {
            unsafe {
                libc::malloc(len * size_of::<T>()) as *mut T
            }
        } else {
            null_mut() as *mut T
        };


        let meta = unsafe {
            let meta =  libc::malloc(1 * size_of::<VectorMeta>()) as *mut VectorMeta;
            (*meta).pos = 0;
            (*meta).len = len;
            (*meta).extend_size = extend_size;
            meta
        };
            


        Vector {
            data,
            meta,
        }
    }


    pub fn push(&mut self, value: T) {
        unsafe {
            if (*self.meta).pos < (*self.meta).len {
                std::ptr::write::<T>(self.data.offset((*self.meta).pos as isize), value);
            } else {
                let new_len = (*self.meta).len + (*self.meta).extend_size;
                let new_data = libc::malloc(new_len * size_of::<T>()) as *mut T;
                for i in 0..(*self.meta).len {
                    *new_data.offset(i as isize) = *self.data.offset(i as isize);
                }

                libc::free(self.data as *mut c_void);

                self.data = new_data;
                (*self.meta).len = new_len;

                std::ptr::write::<T>(self.data.offset((*self.meta).pos as isize), value);
            }
            (*self.meta).pos += 1;
        }
    }


    pub fn as_ptr(&self) -> *mut T {
        self.data
    }


    pub fn len(&self) -> usize {
        unsafe {
            (*self.meta).pos
        }
    }

    pub fn as_slice(&self) -> &mut [T] {
        unsafe {
            from_raw_parts_mut::<T>(self.data, (*self.meta).pos)
        }

    }

    pub fn get(&self, index: usize) -> &mut T {
        unsafe {
            &mut *self.data.offset(index as isize)
        }
    }


    pub fn last(&self) -> &mut T {
        unsafe {
            &mut *self.data.offset(((*self.meta).pos - 1) as isize)
        }
    }


    pub fn free(self) {
        unsafe {
            libc::free(self.data as *mut c_void);
            libc::free(self.meta as *mut c_void);
        }
    }

    pub fn clone(&self) -> Self {
        Vector {
            data: self.data,
            meta: self.meta,
        }
    }
}


impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, i: usize) ->  &Self::Output {
        unsafe {
            &*self.data.offset(i as isize)
        }
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        unsafe {
            &mut *self.data.offset(i as isize)
        }
    }
}
