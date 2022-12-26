use std::os::raw::c_void;
use std::os::raw::c_char;
use std::mem::transmute;
use std::ptr::null_mut;
use crate::dpdk::memory::Array;
// use crate::core::memory::array;


#[derive(Clone)]
pub struct PktBuf {
    pub buf: *mut dpdk_sys::rte_mbuf,
}

impl PktBuf {
    pub fn new() -> Self {
        PktBuf {
            buf: null_mut(),
        }
    }

    pub fn append(&self, len: usize) {
        unsafe {
            dpdk_sys::rte_pktmbuf_append(self.buf, len as u16);
        }
    }

    pub fn get_raw_pkt(&self) -> (*mut u8, usize) {
        unsafe {
            let pkt = transmute::<*mut c_void, *mut u8>((*self.buf).buf_addr);
            let len = (*self.buf).data_len;
            let offset = (*self.buf).data_off;
            (pkt.offset(offset.try_into().unwrap()), len as usize)
        }
    }

    pub fn free(&self, len: u32) {
        unsafe {
            dpdk_sys::rte_pktmbuf_free_bulk(self as *const PktBuf as *mut *mut dpdk_sys::rte_mbuf , len);
        }
    }
}


pub struct PktbufPool {
    mempool: *mut dpdk_sys::rte_mempool,
}

impl PktbufPool {
    pub fn new(len: usize) -> Self {
        let mempool = unsafe {
            dpdk_sys::rte_pktmbuf_pool_create(
                crate::dpdk::common::gen_random_name() .as_ptr() as *mut c_char,
                len as u32,
                256,
                0,
                dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
                dpdk_sys::rte_socket_id().try_into().unwrap()
            )
        };

        PktbufPool {
            mempool,
        }
    }

    pub fn free(&self) {
        unsafe {
            dpdk_sys::rte_mempool_free(self.mempool);
        }
    }

    pub fn alloc_bulk(&self, pktbuf_list: Array<PktBuf>) -> bool {
        unsafe {
            dpdk_sys::rte_pktmbuf_alloc_bulk(
                self.mempool,
                pktbuf_list.as_ptr() as *mut *mut dpdk_sys::rte_mbuf,
                pktbuf_list.len() as u32,
            ) == 0
        }
    }
}
