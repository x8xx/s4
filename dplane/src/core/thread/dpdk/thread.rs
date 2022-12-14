use std::ffi::c_void;

static mut CURRENT_LCORE_ID: u32 = u32::MIN;

pub fn thread_init() {
    unsafe {
        CURRENT_LCORE_ID = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
    }
}

pub fn spawn(func: extern "C" fn(*mut c_void) -> i32, args: *mut c_void) -> bool {
    unsafe {
        let result = dpdk_sys::rte_eal_remote_launch(Some(func), args, CURRENT_LCORE_ID);
        CURRENT_LCORE_ID = dpdk_sys::rte_get_next_lcore(CURRENT_LCORE_ID, 1, 0);
        result == 0
    }
}
