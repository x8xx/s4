use std::ptr::null_mut;
use crate::core::memory::array;

pub struct PktBuf {
    pub bufs: array::Array<*mut dpdk_sys::rte_mbuf>,
}

impl PktBuf {
    pub fn new(len: usize) -> Self {
        PktBuf {
            bufs: array::Array::<*mut dpdk_sys::rte_mbuf>::new(len),
        }
    }
}
