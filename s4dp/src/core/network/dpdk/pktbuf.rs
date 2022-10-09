use std::ptr::null_mut;
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

    pub fn buf_ptr(&self) -> *mut *mut dpdk_sys::rte_mbuf {
        self.buf.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
