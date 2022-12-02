use std::env;
use std::ffi::CString;
use std::os::raw::c_char;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::core::thread::thread::thread_init;


pub fn gen_random_name() -> CString {
    let now = SystemTime::now();
    let unixtime = now.duration_since(UNIX_EPOCH).expect("failed get current time.");
    let name_cstr = CString::new(unixtime.as_nanos().to_string()).unwrap();
    name_cstr
}

pub fn init() -> i32 {
    let cargs: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut dpdk_cargs: Vec<_> = cargs.iter().map(|s| s.as_ptr() as *mut c_char).collect();

    // load shared library (tmp)
    unsafe {
        dpdk_sys::load_rte_virtio_pci_eth_dev();
        dpdk_sys::load_rte_eth_tap();
        dpdk_sys::load_qede_ethdev();
        // dpdk_sys::load_qede_rxtx();
        dpdk_sys::load_rte_mempool_ring();
    }

    unsafe {
        let ret = dpdk_sys::rte_eal_init(dpdk_cargs.len() as i32, dpdk_cargs.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        thread_init(); 

        ret + 1
    }
}


pub fn cleanup() {
    unsafe {
        dpdk_sys::rte_eal_mp_wait_lcore();
        dpdk_sys::rte_eal_cleanup();
    }
}
