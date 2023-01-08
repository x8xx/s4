use std::os::raw::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use crate::core::logger::log::log;
use crate::core::logger::log::debug_log;

pub type RawPktBuf = dpdk_sys::rte_mbuf;

#[derive(Clone)]
pub struct PktBuf {
    pub buf: *mut dpdk_sys::rte_mbuf,
}

impl PktBuf {
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

    pub fn as_raw(&mut self) ->  &mut RawPktBuf {
        unsafe {
            &mut *self.buf
        }
    }

    pub fn free(&self) {
        unsafe {
            debug_log!("Pktbuf Free {:x}", self.buf as i64);
            dpdk_sys::rte_pktmbuf_free(self.buf);
        }
    }
}
