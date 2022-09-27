use std::ops::Index;
use std::ops::IndexMut;
use std::ptr::null_mut;
use std::mem::size_of;

pub struct Array<T> {
    data: *mut T,
    mempool: *mut dpdk_sys::rte_mempool,
}

impl<T> Array<T> {
    pub fn new(len: usize) -> Self {
        let mempool = unsafe {
            dpdk_sys::rte_mempool_create(
                crate::core::helper::dpdk::gen_mempool_name(),
                len as u32,
                size_of::<T>().try_into().unwrap(),
                0,
                0,
                None,
                null_mut(),
                None,
                null_mut(),
                0,
                0
            )
        };

        let data = null_mut();

        Array {
            data,
            mempool,
        }
    }
    
    pub fn head(&self) -> T {
        unsafe {
            *self.data
        }
    }

    pub fn free(self) {
        unsafe {
            dpdk_sys::rte_mempool_free(self.mempool);
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
            &mut *self.data.offset(i as isize) as &mut T
        }
    }
}
