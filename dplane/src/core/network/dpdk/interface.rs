use std::ffi::CStr;
use std::ptr::null_mut;
use std::os::raw::c_char;

use crate::core::network::pktbuf;


#[derive(Clone)]
pub struct Interface {
    pub port_number: u16,
}

impl Interface {
    pub fn new(name: &str) -> Self {
        let port_number = Interface::find_interface_from_name(name);     
        let mempool = unsafe {
            dpdk_sys::rte_pktmbuf_pool_create(
                crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                8192,
                256,
                0,
                dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
                dpdk_sys::rte_socket_id().try_into().unwrap()
            )
        };
        Interface::up(port_number, mempool);

        Interface {
            port_number,
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

            if dpdk_sys::rte_eth_promiscuous_enable(port_number) < 0 {
                panic!("Error rte_eth_promiscuous_enable\n");
            }


            let mut port_info: dpdk_sys::rte_eth_dev_info = Default::default();
            if dpdk_sys::rte_eth_dev_info_get(port_number, &mut port_info as *mut dpdk_sys::rte_eth_dev_info) < 0 {
                println!("failed: get port info");
            } else {
                println!("port{}: max rx queues {}", port_number, port_info.max_rx_queues);
                println!("port{}: max tx queues {}", port_number, port_info.max_tx_queues);
                println!("port{}: min mtu {}", port_number, port_info.min_mtu);
                println!("port{}: max mtu {}", port_number, port_info.max_mtu);
            }

            let mut link : dpdk_sys::rte_eth_link = Default::default();
            if dpdk_sys::rte_eth_link_get(port_number, &mut link as *mut dpdk_sys::rte_eth_link) < 0 {
                println!("failed: get port link info");
            } else {
                println!("port{}: link {}", port_number,  if link.link_status() == 1 { "up" } else { "down" });
                println!("port{}: link spped {}", port_number,  link.link_speed);
            }

            println!("debug {}", dpdk_sys::rte_eth_promiscuous_get(port_number));
            let mut ma_struct = dpdk_sys::rte_ether_addr::default();
            let macaddr = dpdk_sys::rte_eth_macaddrs_get(port_number, &mut ma_struct as *mut dpdk_sys::rte_ether_addr, 6);
            println!("debug mac_addr {:x}:{:x}:{:x}:{:x}:{:x}:{:x}", ma_struct.addr_bytes[0],
                     ma_struct.addr_bytes[1],
                     ma_struct.addr_bytes[2],
                     ma_struct.addr_bytes[3],
                     ma_struct.addr_bytes[4],
                     ma_struct.addr_bytes[5]);
        }
    }

    pub fn rx(&self, pktbuf: &mut pktbuf::PktBuf, len: usize) -> u16 {
        unsafe {
            // dpdk_sys::rte_eth_rx_burst(self.port_number, 0, pktbuf.as_ptr(),  pktbuf.len() as u16);
            dpdk_sys::rte_eth_rx_burst(
                self.port_number,
                0,
                pktbuf as *mut pktbuf::PktBuf as *mut *mut dpdk_sys::rte_mbuf,
                len as u16
            )
        }
    }

    pub fn tx(&self, pktbuf: &mut pktbuf::PktBuf) {
        unsafe {
            // dpdk_sys::rte_eth_tx_burst(self.port_number, 0, pktbuf.as_ptr(),  pktbuf.pkt_count as u16);
            dpdk_sys::rte_eth_tx_burst(
                self.port_number,
                0,
                pktbuf as *mut pktbuf::PktBuf as *mut *mut dpdk_sys::rte_mbuf,
                1
            );
        }
    }
}
