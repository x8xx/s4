use std::ops::Index;
use std::ops::IndexMut;
use std::marker::Send;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;
use std::os::raw::c_char;
use crate::core::memory::heap::Heap;
use crate::core::memory::ptr::Ptr;


#[derive(Clone, Copy)]
pub struct Vector<T> {
    data: *mut T,
    // data_memzone: *const dpdk_sys::rte_memzone,
    meta: *mut VectorMeta,
    // meta_memzone: *const dpdk_sys::rte_memzone,
}

pub struct VectorMeta {
    pub pos: usize,
    pub len: usize,
    pub extend_size: usize, 
}

unsafe impl<T> Send for Vector<T> {}
unsafe impl<T> Sync for Vector<T> {}

impl<T: Copy> Vector<T> {
    pub fn new(len: usize, extend_size: usize) -> Self {
        // let (data_memzone, data) = if len != 0 {
        //     let memzone = unsafe {
        //         dpdk_sys::rte_memzone_reserve(
        //             crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
        //             size_of::<T>() * len,
        //             dpdk_sys::rte_socket_id() as i32,
        //             dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
        //         )
        //     };
        //     (memzone, unsafe { (*memzone).__bindgen_anon_1.addr as *mut T })
        // } else {
        //     (null_mut() as *const dpdk_sys::rte_memzone, null_mut() as *mut T)
        // };


        // let (meta_memzone, meta) = {
        //     let memzone = unsafe {
        //         dpdk_sys::rte_memzone_reserve(
        //             crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
        //             size_of::<VectorMeta>() * 1,
        //             dpdk_sys::rte_socket_id() as i32,
        //             dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
        //         )
        //     };

        //     let meta = unsafe {
        //         let meta = (*memzone).__bindgen_anon_1.addr as *mut VectorMeta;
        //         (*meta).pos = 0;
        //         (*meta).len = len;
        //         (*meta).extend_size = extend_size;
        //         meta
        //     };
            
        //     (memzone, meta)
        // };


        let mut heap = Heap::new().write().unwrap();
        let data = heap.malloc::<T>(len);
        let meta = unsafe {
            let meta = heap.malloc::<VectorMeta>(1) as *mut VectorMeta;
            (*meta).pos = 0;
            (*meta).len = len;
            (*meta).extend_size = extend_size;
            meta
        };

        Vector {
            data,
            // data_memzone,
            meta,
            // meta_memzone,
        }
    }


    pub fn new_manual(data: *mut T, meta: *mut VectorMeta) -> Self {
        Vector {
            data,
            // data_memzone: null_mut(),
            meta,
            // meta_memzone: null_mut(),
        }
    }


    pub fn push(&mut self, value: T) {
        unsafe {
            if (*self.meta).pos < (*self.meta).len {
                std::ptr::write::<T>(self.data.offset((*self.meta).pos as isize), value);
            } else {
                let new_len = (*self.meta).len + (*self.meta).extend_size;
                // let new_memzone = dpdk_sys::rte_memzone_reserve(
                //     crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                //     size_of::<T>() * new_len,
                //     dpdk_sys::rte_socket_id() as i32,
                //     dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
                // );
                // let new_data = (*new_memzone).__bindgen_anon_1.addr as *mut T;
                let new_data = {
                    let mut heap = Heap::new().write().unwrap();
                    heap.malloc(size_of::<T>() * new_len) as *mut T
                };

                for i in 0..(*self.meta).len {
                    *new_data.offset(i as isize) = *self.data.offset(i as isize);
                }

                // if self.data_memzone != null_mut() {
                //     dpdk_sys::rte_memzone_free(self.data_memzone);
                // }

                self.data = new_data;
                // self.data_memzone = new_memzone;
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
            // if self.data_memzone != null_mut() {
            //     dpdk_sys::rte_memzone_free(self.data_memzone);
            // }

            // if self.meta_memzone != null_mut() {
            //     dpdk_sys::rte_memzone_free(self.meta_memzone);
            // }
        }
    }

    pub fn clone(&self) -> Self {
        Vector {
            data: self.data,
            // data_memzone: self.data_memzone,
            meta: self.meta,
            // meta_memzone: self.meta_memzone,
        }
    }
}


impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, i: usize) ->  &T {
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
