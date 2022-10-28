use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::c_void;
use std::marker::PhantomData;

pub struct Ring {
    ring: *mut dpdk_sys::rte_ring,
}

impl Ring {
    pub fn new(len: usize) -> Self {
        #[cfg(feature="dpdk")]
        let ring = {
            unsafe {
                dpdk_sys::rte_ring_create(
                    crate::core::helper::dpdk::gen_random_name(),
                    len as u32,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RING_F_MP_RTS_ENQ | dpdk_sys::RING_F_MC_RTS_DEQ
                    // dpdk_sys::RING_F_MP_HTS_ENQ | dpdk_sys::RING_F_MC_HTS_DEQ
                )
            }
        };

        Ring {
            ring,
        }
    }

    pub fn enqueue<T>(&self, objs: &&mut T, len: usize) -> usize {
        unsafe {
            dpdk_sys::rte_ring_enqueue_burst(
                self.ring,
                objs as *const &mut T as *const *mut c_void,
                len as u32,
                null_mut()
            )
        }.try_into().unwrap()
    }

    pub fn dequeue<T>(&self, objs: &mut &mut T, len: usize) -> usize {
        unsafe {
            dpdk_sys::rte_ring_dequeue_burst(
                self.ring,
                objs as *mut &mut T as *mut *mut c_void,
                len as u32,
                null_mut()
            )
        }.try_into().unwrap()
    }
}


pub struct RingBuf<T> {
    phantom: PhantomData<T>,
    mempool: *mut dpdk_sys::rte_mempool,
}

impl<T> RingBuf<T> {
    pub fn new(len: usize) -> Self {
        let mempool = unsafe {
            dpdk_sys::rte_mempool_create(
                crate::core::helper::dpdk::gen_random_name(),
                len as u32,
                size_of::<T>() as u32,
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

        RingBuf {
            phantom: PhantomData,
            mempool,
        }
    }

    pub fn malloc_bulk(&self, obj: &mut [&T], len: usize) {
        unsafe {
            let obj_ptr = obj as *mut [&T] as *mut *mut T;
            dpdk_sys::rte_mempool_get_bulk(self.mempool, obj_ptr as *mut *mut c_void, len as u32);
        }
    }

    pub fn free_bulk(&self, obj: &[&T], len: usize) {
        unsafe {
            let obj_ptr = obj as *const [&T] as *const *mut T;
            dpdk_sys::rte_mempool_put_bulk(self.mempool, obj_ptr as *const *mut c_void, len as u32);
        }
    }

    pub fn malloc<'a>(&'a self) -> &'a mut  T {
        unsafe {
            let mut obj_ptr: *mut T = null_mut();
            dpdk_sys::rte_mempool_get(self.mempool, &mut obj_ptr as *mut *mut T as *mut *mut c_void);
            &mut *obj_ptr as &mut T
        }
    }

    pub fn free(&self, obj: &mut T) {
        unsafe {
            dpdk_sys::rte_mempool_put(self.mempool, obj as *mut T as *mut c_void);
        }
    }

}
