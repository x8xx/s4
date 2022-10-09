use std::ffi::CStr;
use std::ptr::null_mut;
use crate::core::helper::dpdk::gen_random_name;
use crate::core::network::pktbuf;

pub struct Interface {
    port_number: u16,
}

impl Interface {
    pub fn new(name: &str) -> Self {
        let port_number = Interface::find_interface_from_name(name);     
        let mempool = unsafe {
            dpdk_sys::rte_pktmbuf_pool_create(
                gen_random_name(),
                8192,
                256,
                0,
                dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
                dpdk_sys::rte_socket_id().try_into().unwrap()
            )
        };
        Interface::up(port_number, mempool);

        Interface {
            port_number
        }
    }

    fn find_interface_from_name(name: &str) -> u16 {
        let avail_port_num = unsafe { dpdk_sys::rte_eth_dev_count_avail() };
        if avail_port_num <= 0 {
            panic!("Cannot avail device\n");
        }

        for i in  0..avail_port_num {
            let mut dev_conf: dpdk_sys::rte_eth_dev_info  = Default::default();
            if unsafe { dpdk_sys::rte_eth_dev_info_get(i, &mut dev_conf as *mut _) } != 0 {
                panic!("Cannot get device config port{}\n", i);
            }

            // ex) 0000:06:00.0, net_tap0
            let dev_conf_if_name = unsafe { CStr::from_ptr((*dev_conf.device).name ) }.to_str().unwrap();
            println!("debug {}", dev_conf_if_name);
            if name == dev_conf_if_name {
                return i;
            }
        }
        panic!("Cannot get interface {}\n", name);
    }

    fn up(port_number: u16, mempool: *mut dpdk_sys::rte_mempool) {
        unsafe {
            let port_conf: dpdk_sys::rte_eth_conf = Default::default();
            if dpdk_sys::rte_eth_dev_configure(port_number, 1, 1, &port_conf as *const _) < 0 {
                panic!("Cannot configure device\n");
            }

            let dev_socket_id = dpdk_sys::rte_eth_dev_socket_id(port_number).try_into().unwrap();

            if dpdk_sys::rte_eth_rx_queue_setup(port_number, 0, 1024, dev_socket_id, null_mut(), mempool) < 0 {
                panic!("Error rte_eth_rx_queue_setup\n");

            }

            if dpdk_sys::rte_eth_tx_queue_setup(port_number, 0, 1024, dev_socket_id, null_mut()) < 0 {
                panic!("Error rte_eth_tx_queue_setup\n");

            }

            if dpdk_sys::rte_eth_dev_start(port_number) < 0 {
                panic!("Error rte_eth_dev_start\n");

            }

            dpdk_sys::rte_eth_promiscuous_enable(port_number);
        }
    }

    pub fn rx(&self, pktbuf: &pktbuf::PktBuf) {
        unsafe {
            dpdk_sys::rte_eth_rx_burst(self.port_number, 0, pktbuf.bufs.as_ptr(),  pktbuf.bufs.len() as u16);
        }
    }

    pub fn tx() {

    }
}
