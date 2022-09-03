use std::ffi::c_void;
use std::mem::transmute;
use crate::fib::*;
use crate::dpdk::dpdk_memory;
use crate::dpdk::dpdk_port;
use crate::dpdk::dpdk_eth;


#[repr(C)]
pub struct RxStartArgs<'a> {
    pub if_name: &'a str,
    // pub parser: &'a parser::Parser<'a>,
    pub l1_cache: *mut u8,
    pub lb_filter: *mut u8,
    pub fib_core_rings: &'a [dpdk_memory::Ring],
}


fn next_core() -> usize {
    0
}


pub extern "C" fn rx_start(rx_start_args_ptr: *mut c_void) -> i32 {
    println!("rx_start");

    let rx_start_args = unsafe {&*transmute::<*mut c_void, *mut RxStartArgs>(rx_start_args_ptr)};
    let if_name = &rx_start_args.if_name;
    let l1_cache = &rx_start_args.l1_cache;
    let lb_filter = &rx_start_args.lb_filter;
    let fib_core_rings = &rx_start_args.fib_core_rings;

    println!("create mbuf");
    let pktmbuf = dpdk_memory::create_pktmbuf("mbuf");
    let port_number = dpdk_port::port_init(if_name, pktmbuf);

    println!("create pktprocessor");
    let pp = dpdk_eth::PktProcessor::new(port_number);

    let mut random_next_core = 0;

    println!("start loop");
    loop {
        let rx_count = pp.rx();
        if rx_count <= 0 {
            continue;
        }
        for i in 0..rx_count {
            let pkt = pp.get_packet(i);

            let l1_key = &pkt[0..112];
            let l1_hash = murmurhash3::murmurhash3_x86_32(l1_key, 1);
            println!("l1_hash {}", l1_hash);
            // match l1_cache[l1_hash as usize].compare_key(l1_key) {
            // match l1_cache[0].compare_key(l1_key) {
            //     Some(u8) => continue,
            //     None => 0,
            // };

            let l2_key = &pkt[0..112];
            let l2_hash = murmurhash3::murmurhash3_x86_32(l2_key, 1);
            println!("l2_hash {}", l2_hash);
            // let core_flag = lb_filter[l2_hash as usize];

            // rx_start_args.lb_filter[0] = 0xff;
            // lb_filter[0] = 0xff;
            unsafe {
                let bit_count = core::arch::x86_64::_popcnt64(0xff as i64);
                println!("bit count {}", bit_count);
            }

            // parser
            // create_key(l1 key) l1_cache
            // create_key(l2 key) lb_filter
            // write to ring(packet, l2_key)
            // fib_core_rings[next_core()].enqueue();

            // for i in 0..pkt.len() {
            //     print!("{:x} ", pkt[i]);
            // }
        }
        // pp.tx();
    }

    0
}
