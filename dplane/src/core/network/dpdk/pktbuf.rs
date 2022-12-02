use std::os::raw::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use crate::core::memory::array;


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
            if self.buf == null_mut() {
                return (null_mut(), 0);
            }
            let pkt = transmute::<*mut c_void, *mut u8>((*self.buf).buf_addr);
            let len = (*self.buf).data_len;
            let offset = (*self.buf).data_off;
            (pkt.offset(offset.try_into().unwrap()), len as usize)
        }
    }

    pub fn free(&self) {
        unsafe {
            dpdk_sys::rte_pktmbuf_free(self.buf);
        }
    }
}
