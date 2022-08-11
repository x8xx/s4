use std::ffi::c_void;
use std::mem::transmute;
use crate::fib::*;
use crate::dpdk::dpdk_memory;


#[repr(C)]
pub struct RxStartArgs<'a> {
    pub l1_cache: l1_cache::L1Cache,
    pub lb_filter: lb_filter::LbFilter,
    pub fib_core_rings: &'a [dpdk_memory::Ring],
}


pub extern "C" fn rx_start(rx_start_args_ptr: *mut c_void) -> i32 {
    let rx_start_args = unsafe {&*transmute::<*mut c_void, *mut RxStartArgs>(rx_start_args_ptr)};
    let l1_cache = &rx_start_args.l1_cache;
    let lb_filter = &rx_start_args.lb_filter;
    // let fib_core_rings = &rx_start_args.fib_core_rings;

    // let pktmbuf = dpdk_create_pktmbuf("mbuf");

    // while true {

    // }


    println!("rx_start");
    0
}
