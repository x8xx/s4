use std::env;
use std::os::raw::c_char;
use std::ffi::CString;


pub fn init() -> i32 {
    let cargs: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut dpdk_cargs: Vec<_> = cargs.iter().map(|s| s.as_ptr() as *mut c_char).collect();

    // load shared library
    unsafe {
        dpdk_sys::output_test_log();
        dpdk_sys::load_rte_eth_tap();
    }

    unsafe {
        let ret = dpdk_sys::rte_eal_init(dpdk_cargs.len() as i32, dpdk_cargs.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        ret + 1
    }
}


pub fn cleanup() {
    unsafe {
        dpdk_sys::rte_eal_mp_wait_lcore();
        dpdk_sys::rte_eal_cleanup();
    }
}
