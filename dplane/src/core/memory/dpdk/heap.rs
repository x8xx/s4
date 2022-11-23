use std::mem::size_of;
use std::ptr::null_mut;
use std::os::raw::c_char;
use crate::core::memory::array::Array;


pub struct Heap {
    data: *mut u8,
    memzone: *const dpdk_sys::rte_memzone,
    size: usize,
    next_pos: isize,
}


impl Heap {
    pub fn new(size: usize) -> Self {
        let (memzone, data) = if size != 0 {
            let memzone = unsafe {
                dpdk_sys::rte_memzone_reserve(
                    // crate::core::helper::dpdk::gen_random_name(),
                    crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                    size_of::<u8>() * size,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
                )
            };
            (memzone, unsafe { (*memzone).__bindgen_anon_1.addr as *mut u8 })
        } else {
            (null_mut() as *const dpdk_sys::rte_memzone, null_mut() as *mut u8)
        };

        Heap {
            data,
            memzone,
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
            dpdk_sys::rte_memzone_free(self.memzone);
        }
    }
}
