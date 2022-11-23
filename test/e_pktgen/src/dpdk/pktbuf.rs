use std::os::raw::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
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