use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::ptr::slice_from_raw_parts_mut;
use std::mem::size_of;
use std::mem::transmute;


pub struct Ring {
    rte_ring: *mut dpdk_sys::rte_ring,
}


impl Ring {
    pub fn  new(name: &str, size: u32) -> Self {
        let name_cstr = CString::new(name).unwrap();
        Ring {
            rte_ring: unsafe {
                dpdk_sys::rte_ring_create(
                    name_cstr.as_ptr() as *mut c_char,
                    size,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RING_F_SP_ENQ | dpdk_sys::RING_F_SC_DEQ
                )
            }
        }

    }

    pub fn enqueue(&self, receive_ptr: *const *mut c_void, obj_len: u32) -> u32 {
        unsafe {
            dpdk_sys::rte_ring_enqueue_burst(self.rte_ring, receive_ptr, obj_len, null_mut())
        }
    }


    pub fn dequeue(&self, obj_ptr: *mut *mut c_void, obj_len: u32) -> u32 {
        unsafe {
            dpdk_sys::rte_ring_dequeue_burst(self.rte_ring, obj_ptr, obj_len, null_mut())
        }
    }
}


pub fn create_pktmbuf(name: &str) -> *mut dpdk_sys::rte_mempool {
    let cstr_mbuf_pool = CString::new(name).unwrap();
    unsafe {
        dpdk_sys::rte_pktmbuf_pool_create(
            cstr_mbuf_pool.as_ptr() as *mut c_char,
            8192,
            256,
            0,
            dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
            dpdk_sys::rte_socket_id().try_into().unwrap()
        )
    }
}


pub fn malloc<T>(name: &str, size: u32) -> *mut T {
    let name_cstr = CString::new(name).unwrap();
    unsafe {
        let mempool = dpdk_sys::rte_mempool_create(
            name_cstr.as_ptr() as *mut c_char,
            size,
            size_of::<T>().try_into().unwrap(),
            0,
            0,
            None,
            null_mut(),
            None,
            null_mut(),
            0,
            0
        );
        transmute::<*mut c_void, *mut T>((*mempool).__bindgen_anon_1.pool_data)
    }
}


pub fn free<T>(data: *mut T) -> bool {
    // rte_mempool_free
    true
}
