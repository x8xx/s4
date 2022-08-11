pub mod fib;
pub mod rx;
use std::ffi::c_void;


pub fn remote_launch_rx(lcore_id: u32, rx_start_args: &mut rx::RxStartArgs) -> bool {
    let rx_start_args_ptr = rx_start_args as *mut rx::RxStartArgs as *mut c_void;
    unsafe { dpdk_sys::rte_eal_remote_launch(Some(rx::rx_start), rx_start_args_ptr, lcore_id) == 0 }
}


pub fn remote_launch_fib(lcore_id: u32, fib_start_args: &mut fib::FibStartArgs) -> bool {
    let fib_start_args_ptr = fib_start_args as *mut fib::FibStartArgs as *mut c_void;
    unsafe { dpdk_sys::rte_eal_remote_launch(Some(fib::fib_start), fib_start_args_ptr, lcore_id) == 0 }
}
