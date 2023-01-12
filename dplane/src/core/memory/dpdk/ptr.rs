use crate::core::memory::heap::Heap;
use std::ptr;
use std::cmp::PartialEq;
use std::ops::Deref;
use std::ops::DerefMut;


#[derive(Clone, Copy)]
pub struct Ptr<T> {
    data_ptr: *mut T,
}

impl<T> Ptr<T> {
    pub fn new(data: T) -> Self {
        let data_ptr = {
            let mut heap = Heap::new().write().unwrap();
            heap.malloc::<T>(1)
        };
        unsafe {
            ptr::write::<T>(data_ptr, data);
        }
        Ptr {
            data_ptr,
        }
    }
}


impl<T> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data_ptr as u64 == other.data_ptr as u64
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &*self.data_ptr
        }
    }
}


impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            &mut *self.data_ptr
        }
    }
}
