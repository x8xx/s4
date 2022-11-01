use std::os::raw::c_void;
use std::mem::transmute;
use std::slice::from_raw_parts;
use crate::core::memory::array;

pub struct PktBuf {
    pub buf: array::Array<*mut dpdk_sys::rte_mbuf>,
    len: usize,
}

impl PktBuf {
    pub fn new(len: usize) -> Self {
        PktBuf {
            buf: array::Array::<*mut dpdk_sys::rte_mbuf>::new(len),
            len,
        }
    }

    pub fn as_ptr(&self) -> *mut *mut dpdk_sys::rte_mbuf {
        self.buf.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get_raw_pkt(&self, index: usize) -> (*mut u8, usize) {
        unsafe {
            let pkt = unsafe { transmute::<*mut c_void, *mut u8>((*self.buf[index]).buf_addr) };
            let len = (*self.buf[index]).data_len;
            let offset = (*self.buf[index]).data_off;
            // from_raw_parts(pkt.offset(offset.try_into().unwrap()), (offset + len) as usize)
            (pkt.offset(offset.try_into().unwrap()), len as usize)
        }
    }

    pub fn free(&self) {

    }
}
