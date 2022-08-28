use std::ffi::c_void;
use std::mem::transmute;
use crate::dpdk::dpdk_memory;
use crate::fib::*;


#[repr(C)]
// pub struct FibStartArgs<'a> {
pub struct FibStartArgs {
    // fib_core_ring: &'a dpdk_memory::Ring,
}


#[repr(C)]
pub struct FibRingArgs<'a> {
    packet_buf: &'a [u8],
    // parser: &'a [parser::Header<'a>],
}


pub extern "C" fn fib_start(fib_start_args_ptr: *mut c_void) -> i32 {
    println!("fib_start");
    let fib_start_args = unsafe {&*transmute::<*mut c_void, *mut FibStartArgs>(fib_start_args_ptr)};
    // let  fib_core_ring = &fib_start_args.fib_core_ring;

    loop {
        
    }
    0
}
