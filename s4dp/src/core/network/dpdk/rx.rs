use std::ptr::null_mut;

pub struct Rx {
    port_number: u16,
    buf: [*mut dpdk_sys::rte_mbuf; 32],
}

impl Rx {
    pub fn new(port_number: u16) -> Self {
        Rx {
            port_number,
            buf: [null_mut(), 32],
        }
    
    }

    pub fn rx(&self) -> u16 {
        unsafe { 
            dpdk_sys::rte_eth_rx_burst(self.port_number, 0, self.buf.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, 32)
        }
    }
}
