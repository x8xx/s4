// TODO

use crate::core::memory::array::Array;


#[derive(Clone)]
pub struct PktBuf {
    pub buf: Array<u8>,
}

impl PktBuf {
    pub fn new() -> Self {
        PktBuf {
            buf: Array::new(0),
        }
    }

    pub fn get_raw_pkt(&self) -> (*mut u8, usize) {
        (self.buf.as_ptr(), 0)
        // unsafe {
        //     let pkt = transmute::<*mut c_void, *mut u8>((*self.buf).buf_addr);
        //     let len = (*self.buf).data_len;
        //     let offset = (*self.buf).data_off;
        //     (pkt.offset(offset.try_into().unwrap()), len as usize)
        // }
    }

    pub fn free(&self) {
        // unsafe {
        //     dpdk_sys::rte_pktmbuf_free(self.buf);
        // }
    }
}
