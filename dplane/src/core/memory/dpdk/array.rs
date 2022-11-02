use std::ops::Index;
use std::ops::IndexMut;
// use std::iter::Iterator;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;
use std::os::raw::c_char;


pub struct Array<T> {
    data: *mut T,
    memzone: *const dpdk_sys::rte_memzone,
    len: usize,
}

impl<T> Array<T> {
    pub fn new(len: usize) -> Self {
        let (memzone, data) = if len != 0 {
            let memzone = unsafe {
                dpdk_sys::rte_memzone_reserve(
                    // crate::core::helper::dpdk::gen_random_name(),
                    crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                    size_of::<T>() * len,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
                )
            };
            (memzone, unsafe { (*memzone).__bindgen_anon_1.addr as *mut T })
        } else {
            (null_mut() as *const dpdk_sys::rte_memzone, null_mut() as *mut T)
        };

        Array {
            data,
            memzone,
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

    pub fn free(self) {
        unsafe {
            dpdk_sys::rte_memzone_free(self.memzone);
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

// impl<T> Iterator for Array<T> {
//     type Item = *mut T;

//     fn next(&mut self) -> Option<*mut T> {
//         if self.iter_pos == self.len as isize {
//             return None;
//         }
//         let obj = unsafe { &mut *self.data.offset(self.iter_pos) };
//         self.iter_pos += 1;
//         Some(obj)
//     }

// }
