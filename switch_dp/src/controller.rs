use std::collections::HashMap;
use crate::config::*;
use crate::dpdk::dpdk_memory;
use crate::worker::*;
use crate::fib::cache::*;
use crate::fib::parser::*;


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


// main core
pub fn controller_start(switch_config: &SwitchConfig) {
    /* ------------------------
     * parser
     * --------------------------*/
    // let dp_config_json = get_sample_dp_config();
    // let dp_config = DataPlaneConfig::new(&dp_config_json); 
    // let mut parser_hashmap: HashMap<String, &Parser> = HashMap::new();
    // let mut hdr_hashmap: HashMap<String, &Header> = HashMap::new();
    // let parser = create_parser(&dp_config, &mut parser_hashmap, &mut hdr_hashmap, "start", 0);


    /* ------------------------
     * rx core
     * --------------------------*/
    let l1_cache_len = 256;
    let l1_cache = dpdk_memory::malloc::<CacheElement>("l1_cache".to_string(), l1_cache_len);
    println!("{}", std::mem::size_of::<usize>());
    let l1_key_max_len = 48;
    for i in 0..l1_cache_len {
        let key_mempool = dpdk_memory::malloc::<u8>(format!("l1_cache_key_{}", i), l1_key_max_len);
        l1_cache[i as usize].clean(key_mempool);
    }
    println!("{}", l1_cache.len());

    let lb_filter_len = 65535;
    let lb_filter = dpdk_memory::malloc::<u8>("lb_filter".to_string(), lb_filter_len);
    lb_filter[0] = 0xff;

    let mut fib_core_rings = Vec::new();
    fib_core_rings.push(dpdk_memory::Ring::new("fib1", 4096));
    fib_core_rings.push(dpdk_memory::Ring::new("fib2", 4096));

    let mut rx_start_args = rx::RxStartArgs {
        if_name: &switch_config.if_name,
        // parser: &parser,
        l1_cache,
        lb_filter,
        fib_core_rings: &fib_core_rings,
    };


    /* ------------------------
     * fib core
     * --------------------------*/
    let mut fib_start_args =fib::FibStartArgs {

    };

    allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args);
}
