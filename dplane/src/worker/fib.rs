use std::ffi::c_void;
use std::mem::transmute;
use crate::dpdk::dpdk_memory;
use crate::fib::*;


#[repr(C)]
pub struct FibStartArgs<'a> {
    pub fib_core_ring: &'a dpdk_memory::Ring,
    pub core_id: u8,
}


#[repr(C)]
pub struct FibRingArgs<'a> {
    packet_buf: &'a [u8],
    // parser: &'a [parser::Header<'a>],
}


pub extern "C" fn fib_start(fib_start_args_ptr: *mut c_void) -> i32 {
    println!("fib_start");
    let fib_start_args = unsafe {&*transmute::<*mut c_void, *mut FibStartArgs>(fib_start_args_ptr)};
    let fib_core_ring = fib_start_args.fib_core_ring;
    let core_id = fib_start_args.core_id;

    if core_id == 1 {
        return 0;
    }

    let pkt = dpdk_memory::malloc::<*mut u8>(format!("fib_pkt_{}", core_id), 1500);

    loop {
        if fib_core_ring.dequeue::<u8>(pkt) == 0 {
            println!("ok {:p} {}", pkt, core_id);
        }
    }
    0
}
