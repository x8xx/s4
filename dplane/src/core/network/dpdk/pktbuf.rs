use std::os::raw::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use crate::core::memory::array;


pub struct PktBuf {
    // pub buf: array::Array<*mut dpdk_sys::rte_mbuf>,
    pub buf: *mut dpdk_sys::rte_mbuf,
    // pub id: usize,
    // pub pkt_count: u16,
    // len: usize,
}

impl PktBuf {
    // pub fn new(len: usize) -> Self {
    pub fn new() -> Self {
        PktBuf {
            buf: null_mut(),
            // id: 0,
            // buf: array::Array::<*mut dpdk_sys::rte_mbuf>::new(len),
            // pkt_count: 0,
            // len,
        }
    }

    // pub fn as_ptr(&self) -> *mut *mut dpdk_sys::rte_mbuf {
    //     self.buf.as_ptr()
    // }

    // pub fn len(&self) -> usize {
    //     self.len
    // }

    // pub fn get_raw_pkt(&self, index: usize) -> (*mut u8, usize) {
    pub fn get_raw_pkt(&self) -> (*mut u8, usize) {
        unsafe {
            let pkt = unsafe { transmute::<*mut c_void, *mut u8>((*self.buf).buf_addr) };
            let len = (*self.buf).data_len;
            let offset = (*self.buf).data_off;
            (pkt.offset(offset.try_into().unwrap()), len as usize)
        }
    }

    pub fn free(&mut self) {
        unsafe {
            // dpdk_sys::rte_pktmbuf_free_bulk(self.buf.as_ptr(), self.pkt_count as u32);
            // self.pkt_count = 0;
        }
    }
}
