use std::ptr::null_mut;
use std::os::raw::c_void;
use std::mem::transmute;
use std::slice::from_raw_parts;


pub struct PktProcessor {
    buf: [*mut  dpdk_sys::rte_mbuf; 32],
    port_number: u16,
}

impl PktProcessor {
    pub fn new(port_number: u16) -> Self {
        PktProcessor { 
            buf: [null_mut(); 32],
            port_number,
        }
    }


    pub fn rx(&self) -> u16 {
        unsafe { dpdk_sys::rte_eth_rx_burst(self.port_number, 0, self.buf.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, 32) }
    }


    pub fn get_packet(&self, index: u16) -> &[u8] {
        let i = index as usize;
        let pkt = unsafe { transmute::<*mut c_void, *mut u8>((*self.buf[i]).buf_addr) };
        let data_len = unsafe { (*self.buf[i]).data_len };
        let data_off = unsafe { (*self.buf[i]).data_off };
        unsafe { from_raw_parts(pkt.offset(data_off.try_into().unwrap()), (data_off + data_len) as usize) }
    }


    pub fn tx(&self) -> u16 {
        self.free();
        0
    }


    pub fn free(&self) {
        unsafe {
            dpdk_sys::rte_pktmbuf_free(self.buf[0]);
        }
    }

}
