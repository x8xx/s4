use std::ops::Index;
use std::ops::IndexMut;
use std::mem::size_of;

pub struct Array<T> {
    data: *mut T,
    memzone: *const dpdk_sys::rte_memzone,
}

impl<T> Array<T> {
    pub fn new(len: usize) -> Self {
        let memzone = unsafe {
            dpdk_sys::rte_memzone_reserve(
                crate::core::helper::dpdk::gen_random_name(),
                size_of::<T>() * len,
                dpdk_sys::rte_socket_id() as i32,
                dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
            )
        };

        let data = unsafe { (*memzone).__bindgen_anon_1.addr as *mut T };

        Array {
            data,
            memzone,
        }
    }

    pub fn free(self) {
        unsafe {
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
