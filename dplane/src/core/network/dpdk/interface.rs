use std::ffi::CStr;
use std::ptr::null_mut;
use std::os::raw::c_char;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf;


#[derive(Clone)]
pub struct Interface {
    pub port: u16,
    pub queue: u16,
}


impl Interface {
    pub fn init(name: &str, mut rx_queues: u16, mut tx_queues: u16) -> (u16, u16, u16) {
        let port = Interface::find_interface_from_name(name);     
        let (max_rx_queues, max_tx_queues) = unsafe {
            let mut port_info: dpdk_sys::rte_eth_dev_info = Default::default();
            if dpdk_sys::rte_eth_dev_info_get(port, &mut port_info as *mut dpdk_sys::rte_eth_dev_info) < 0 {
                panic!("failed: get port info");
            }
            (port_info.max_rx_queues, port_info.max_tx_queues)
        };

        rx_queues = if max_rx_queues < rx_queues { max_rx_queues } else { rx_queues };
        tx_queues = if max_tx_queues < tx_queues { max_tx_queues } else { tx_queues };
        Interface::up(port, rx_queues, tx_queues);

        (port, rx_queues, tx_queues)

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

    fn up(port: u16, max_rx_queues: u16, max_tx_queues: u16) {
        unsafe {
            let mut port_conf: dpdk_sys::rte_eth_conf = Default::default();
            // port_conf.rxmode.mq_mode = dpdk_sys::rte_eth_rx_mq_mode_RTE_ETH_MQ_RX_RSS;
            // port_conf.rx_adv_conf.rss_conf.rss_hf =  0x180d34;

            if dpdk_sys::rte_eth_dev_configure(port, max_rx_queues, max_tx_queues, &port_conf as *const _) < 0 {
                panic!("Cannot configure device\n");
            }

            let dev_socket_id = dpdk_sys::rte_eth_dev_socket_id(port).try_into().unwrap();

            for i in 0..max_rx_queues {
                let mempool = dpdk_sys::rte_pktmbuf_pool_create(
                    crate::core::helper::dpdk::gen_random_name() .as_ptr() as *mut c_char,
                    // number of elements
                    // 8192,
                    // 65536,
                    262144,

                    // number of cache elements
                    // 0,
                    // 256,
                    512,

                    0,
                    dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
                    dpdk_sys::rte_socket_id().try_into().unwrap()
                );

                if dpdk_sys::rte_eth_rx_queue_setup(port, i, 1024, dev_socket_id, null_mut(), mempool) < 0 {
                // if dpdk_sys::rte_eth_rx_queue_setup(port, i, 32768, dev_socket_id, null_mut(), mempool) < 0 {
                    panic!("Error rte_eth_rx_queue_setup Port{} Queue{}\n", port, i);
                }
            }

            for i in 0..max_tx_queues {
                if dpdk_sys::rte_eth_tx_queue_setup(port, i, 1024, dev_socket_id, null_mut()) < 0 {
                // if dpdk_sys::rte_eth_tx_queue_setup(port, i, 32768, dev_socket_id, null_mut()) < 0 {
                    panic!("Error rte_eth_tx_queue_setup Port{} Queue{}\n", port, i);
                }
            }


            if dpdk_sys::rte_eth_dev_start(port) < 0 {
                panic!("Error rte_eth_dev_start\n");

            }

            if dpdk_sys::rte_eth_promiscuous_enable(port) < 0 {
                panic!("Error rte_eth_promiscuous_enable\n");
            }


            let mut port_info: dpdk_sys::rte_eth_dev_info = Default::default();
            if dpdk_sys::rte_eth_dev_info_get(port, &mut port_info as *mut dpdk_sys::rte_eth_dev_info) < 0 {
                println!("failed: get port info");
            } else {
                println!("port{}: max rx queues {}", port, port_info.max_rx_queues);
                println!("port{}: max tx queues {}", port, port_info.max_tx_queues);
                println!("port{}: min mtu {}", port, port_info.min_mtu);
                println!("port{}: max mtu {}", port, port_info.max_mtu);
            }

            let mut link : dpdk_sys::rte_eth_link = Default::default();
            if dpdk_sys::rte_eth_link_get(port, &mut link as *mut dpdk_sys::rte_eth_link) < 0 {
                println!("failed: get port link info");
            } else {
                println!("port{}: link {}", port,  if link.link_status() == 1 { "up" } else { "down" });
                println!("port{}: link spped {}", port,  link.link_speed);
            }

            println!("debug {}", dpdk_sys::rte_eth_promiscuous_get(port));
            let mut ma_struct = dpdk_sys::rte_ether_addr::default();
            let macaddr = dpdk_sys::rte_eth_macaddrs_get(port, &mut ma_struct as *mut dpdk_sys::rte_ether_addr, 6);
            println!("debug mac_addr {:x}:{:x}:{:x}:{:x}:{:x}:{:x}", ma_struct.addr_bytes[0],
                     ma_struct.addr_bytes[1],
                     ma_struct.addr_bytes[2],
                     ma_struct.addr_bytes[3],
                     ma_struct.addr_bytes[4],
                     ma_struct.addr_bytes[5]);
        }
    }

    /**
     * debug
     */
    pub fn debug_show_info(&self) {
        if cfg!(feature="log-level-debug") {
            unsafe {
                let mut port_info: dpdk_sys::rte_eth_dev_info = Default::default();
                if dpdk_sys::rte_eth_dev_info_get(self.port, &mut port_info as *mut dpdk_sys::rte_eth_dev_info) < 0 {
                    println!("failed: get port info");
                } else {
                    println!("port{}: max rx queues {}", self.port, port_info.max_rx_queues);
                    println!("port{}: max tx queues {}", self.port, port_info.max_tx_queues);
                    println!("port{}: min mtu {}", self.port, port_info.min_mtu);
                    println!("port{}: max mtu {}", self.port, port_info.max_mtu);
                }

                let mut link : dpdk_sys::rte_eth_link = Default::default();
                if dpdk_sys::rte_eth_link_get(self.port, &mut link as *mut dpdk_sys::rte_eth_link) < 0 {
                    println!("failed: get port link info");
                } else {
                    println!("port{}: link {}", self.port,  if link.link_status() == 1 { "up" } else { "down" });
                    println!("port{}: link spped {}", self.port,  link.link_speed);
                }

                let mut ma_struct = dpdk_sys::rte_ether_addr::default();
                let macaddr = dpdk_sys::rte_eth_macaddrs_get(self.port, &mut ma_struct as *mut dpdk_sys::rte_ether_addr, 6);
                println!("debug mac_addr {:x}:{:x}:{:x}:{:x}:{:x}:{:x}", ma_struct.addr_bytes[0],
                         ma_struct.addr_bytes[1],
                         ma_struct.addr_bytes[2],
                         ma_struct.addr_bytes[3],
                         ma_struct.addr_bytes[4],
                         ma_struct.addr_bytes[5]);
            }
        }
        // unsafe {
        //     println!("debug rte_mempool_get_count {}", dpdk_sys::rte_mempool_in_use_count(self.mempool));
        // }
    }

    pub fn rx(&self, pktbuf: &Array<pktbuf::PktBuf>, len: usize) -> u16 {
        unsafe {
            // dpdk_sys::rte_eth_rx_burst(self.port, 0, pktbuf.as_ptr(),  pktbuf.len() as u16);
            dpdk_sys::rte_eth_rx_burst(
                self.port,
                self.queue,
                pktbuf.as_ptr() as *mut pktbuf::PktBuf as *mut *mut dpdk_sys::rte_mbuf,
                len as u16
            )
        }
    }

    pub fn tx(&self, pktbuf: &Array<&mut pktbuf::RawPktBuf>, len: usize) -> u16 {
        unsafe {
            dpdk_sys::rte_eth_tx_burst(
                self.port,
                self.queue,
                pktbuf.as_ptr() as *mut &mut pktbuf::RawPktBuf as *mut *mut dpdk_sys::rte_mbuf,
                len as u16
            )
        }
    }
}
