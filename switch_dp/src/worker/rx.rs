use std::ffi::c_void;
use std::mem::transmute;
use crate::fib::*;
use crate::dpdk::dpdk_memory;
use crate::dpdk::dpdk_port;
use crate::dpdk::dpdk_eth;


#[repr(C)]
pub struct RxStartArgs<'a> {
    pub if_name: &'a str,
    pub l1_cache: l1_cache::L1Cache,
    pub lb_filter: lb_filter::LbFilter,
    pub fib_core_rings: &'a [dpdk_memory::Ring],
}


pub extern "C" fn rx_start(rx_start_args_ptr: *mut c_void) -> i32 {
    println!("rx_start");

    let rx_start_args = unsafe {&*transmute::<*mut c_void, *mut RxStartArgs>(rx_start_args_ptr)};
    let if_name = &rx_start_args.if_name;
    let l1_cache = &rx_start_args.l1_cache;
    let lb_filter = &rx_start_args.lb_filter;
    let fib_core_rings = &rx_start_args.fib_core_rings;

    let pktmbuf = dpdk_memory::create_pktmbuf("mbuf");
    let port_number = dpdk_port::port_init(if_name, pktmbuf);

    let pp = dpdk_eth::PktProcessor::new(port_number);
    loop {
        let rx_count = pp.rx();
        if rx_count <= 0 {
            continue;
        }
        for i in 0..rx_count {
            let pkt = pp.get_packet(i);
            for i in 0..pkt.len() {
                print!("{:x} ", pkt[i]);
            }
        }
        // pp.tx();
    }

    // let mut pkts: [*mut dpdk_sys::rte_mbuf; 32] = [null_mut(); 32];
    // while true {
    //     let tap_rx = dpdk_sys::rte_eth_rx_burst(port_number, 0, pkts.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, 32);
    //     if tap_rx <= 0 {
    //         continue;
    //     }
    //     println!("recv: {}", tap_rx);
    //     for i in 0..tap_rx {
    //         let pkt = std::mem::transmute::<*mut  std::os::raw::c_void, *mut u8>((*pkts[i as usize]).buf_addr);
    //         let len = (*pkts[i as usize]).data_len;
    //         let off = (*pkts[i as usize]).data_off;
    //         println!("{}: len{}, off{}", i, len, off);
    //         for j in off..len+off {
    //             print!("{:x} ", *pkt.offset(j.try_into().unwrap()));
    //         }
    //         println!("");
    //         // dpdk_sys::rte_pktmbuf_free(pkts[i as usize]);
    //     }
    //     // let tap_tx = dpdk_sys::rte_eth_tx_burst(1, 0, pkts.as_ptr() as *mut *mut dpdk_sys::rte_mbuf, tap_rx);
    //     // println!("send: {}", tap_tx);
    // }
    0
}
