use std::ptr::null_mut;
use std::ffi::CStr;


pub fn port_init(port_name: &str, pktmbuf: *mut dpdk_sys::rte_mempool) -> u16 {
    let avail_port_num = unsafe { dpdk_sys::rte_eth_dev_count_avail() };
    if avail_port_num <= 0 {
        panic!("Cannot avail device\n");
    }

    println!("test: {}", avail_port_num);
    for i in  0..avail_port_num {
        let mut dev_conf: dpdk_sys::rte_eth_dev_info  = Default::default();
        if unsafe { dpdk_sys::rte_eth_dev_info_get(i, &mut dev_conf as *mut _) } != 0 {
            panic!("Cannot get device config port{}\n", i);
        }

        // ex) 0000:06:00.0, net_tap0
        let dev_conf_if_name = unsafe { CStr::from_ptr((*dev_conf.device).name ) }.to_str().unwrap();
        println!("debug {}", dev_conf_if_name);
        if port_name == dev_conf_if_name {
            port_start(i, pktmbuf);
            return i;
        }
    }
    panic!("Cannot find port {}", port_name);
}


fn port_start(port_number: u16, pktmbuf: *mut dpdk_sys::rte_mempool) {
    unsafe {
        let port_conf: dpdk_sys::rte_eth_conf = Default::default();
        if dpdk_sys::rte_eth_dev_configure(port_number, 1, 1, &port_conf as *const _) < 0 {
            panic!("Cannot configure device\n");
        }

        let dev_socket_id = dpdk_sys::rte_eth_dev_socket_id(port_number).try_into().unwrap();

        if dpdk_sys::rte_eth_rx_queue_setup(port_number, 0, 1024, dev_socket_id, null_mut(), pktmbuf) < 0 {
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
