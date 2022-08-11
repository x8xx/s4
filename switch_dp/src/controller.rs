use std::thread;
use std::time::Duration;

use crate::config::*;
use crate::dpdk::dpdk_memory;
use crate::fib::lb_filter;
use crate::worker::*;
use crate::fib::cache::*;
use crate::fib::l1_cache::*;
use crate::fib::lb_filter::*;
// use crate::fib::l2_cache;
// use crate::fib::l3_cache;


fn allocate_core_to_worker(switch_config: &SwitchConfig,
                            rx_start_args: &mut rx::RxStartArgs,
                            fib_start_args: &mut fib::FibStartArgs) -> bool {
    let mut unallocated_rx_core = switch_config.rx_cores;
    let mut unallocated_fib_core = switch_config.fib_cores;
    unsafe {
        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            if unallocated_rx_core > 0 {
                if !remote_launch_rx(lcore_id, rx_start_args) {
                    panic!("Failed start rx worker");
                }
                unallocated_rx_core -= 1;
            } else if unallocated_fib_core > 0 {
                if !remote_launch_fib(lcore_id, fib_start_args) {
                    panic!("Failed start fib worker");
                }
                unallocated_fib_core -= 1;
            }
            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }
    }
    
    !(unallocated_rx_core > 0 || unallocated_fib_core > 0)
}


pub fn controller_start(switch_config: &SwitchConfig) {
    let l1_cache_len = 4096;
    let l1_cache_element_ptr = dpdk_memory::malloc::<CacheElement>("l1_cache", l1_cache_len);
    let l1_cache = L1Cache::new(l1_cache_element_ptr, l1_cache_len, 1);

    let lb_filter_len = 65535;
    let lb_filter_ptr = dpdk_memory::malloc::<u8>("lb_filter", lb_filter_len);
    let lb_filter = LbFilter::new(lb_filter_ptr, lb_filter_len);

    let mut fib_core_rings = Vec::new();
    fib_core_rings.push(dpdk_memory::Ring::new("fib1", 4096));
    fib_core_rings.push(dpdk_memory::Ring::new("fib2", 4096));


    let mut rx_start_args = rx::RxStartArgs {
        l1_cache,
        lb_filter,
        fib_core_rings: &fib_core_rings,
    };
    let mut fib_start_args =fib::FibStartArgs {};

    allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args);


    // ===========================
    // thread test
    // ===========================
    // let handle = thread::spawn(|| {
    // for i in 1..10 {
    //     println!("hi number {} from the spawned thread!", i);
    //     thread::sleep(Duration::from_millis(1));
    //     }
    // });

    // for i in 1..5 {
    //     println!("hi number {} from the main thread!", i);
    //     thread::sleep(Duration::from_millis(1));
    // }
    // handle.join().unwrap();
}
