mod worker;
use std::env;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::CString;
use getopts::Options;

// extern "C" fn lcore_hello(_: *mut c_void) -> i32 {
//     unsafe {
//         println!("hello from core {}", dpdk_sys::rte_lcore_id());
//     }
//     0
// }

struct SwitchConfig {
    rx_cores: u8,
    fib_cores: u8,
}


fn parse_switch_args(args: &[&str]) -> SwitchConfig {
    let mut opts = Options::new();
    opts.optopt("r", "rx-cores", "number of rx cores to allocate", "");
    opts.optopt("f", "fib-cores", "number of fib cores to allocate", "");

    let matches = opts.parse(args).unwrap();
    let rx_cores: u8 = matches.opt_get::<u8>("r").unwrap().unwrap();
    let fib_cores: u8 = matches.opt_get::<u8>("f").unwrap().unwrap();

    SwitchConfig { rx_cores, fib_cores }
}


fn remote_launch_worker(switch_config: &SwitchConfig) {
    let mut unallocated_rx_core = switch_config.rx_cores;
    let mut unallocated_fib_core = switch_config.fib_cores;
    unsafe {
        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            if unallocated_rx_core > 0 {
                dpdk_sys::rte_eal_remote_launch(Some(worker::rx::rx_start), null_mut(), lcore_id);
                unallocated_rx_core -= 1;
            }
            
            if unallocated_fib_core > 0 {
                dpdk_sys::rte_eal_remote_launch(Some(worker::fib::fib_start), null_mut(), lcore_id);
                unallocated_fib_core -= 1;
            }

            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }
    }
}


fn main() {
    // init
    let cargs: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    let mut dpdk_cargs: Vec<_> = cargs.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    let switch_args_start_index = unsafe {
        let ret = dpdk_sys::rte_eal_init(dpdk_cargs.len() as i32, dpdk_cargs.as_mut_ptr());
        if ret < 0 {
            panic!("Cannot init EAL\n");
        }

        (ret + 1) as usize
    };

    let args: Vec<&str> = cargs.iter().map(|cs| cs.to_str().unwrap()).collect();
    let switch_args: &[&str] = &args[switch_args_start_index..];
    let switch_config = parse_switch_args(switch_args);


    // load shared lib
    unsafe {
        dpdk_sys::output_test_log();
        dpdk_sys::load_rte_eth_tap();
    }


    // port check
    let avail_port_num = unsafe { dpdk_sys::rte_eth_dev_count_avail() };
    if avail_port_num <= 0 {
        panic!("Cannot avail device\n");
    }
    println!("port {}", avail_port_num);


    // start up worker
    remote_launch_worker(&switch_config);
    println!("main_start");

    unsafe {
        // allocate pktmbuf
        // let cstr_mbuf_pool = CString::new("mbuf_pool").unwrap();
        // let mut buf = dpdk_sys::rte_pktmbuf_pool_create(
        //     cstr_mbuf_pool.as_ptr() as *mut c_char,
        //     8192,
        //     256,
        //     0,
        //     dpdk_sys::RTE_MBUF_DEFAULT_BUF_SIZE.try_into().unwrap(),
        //     dpdk_sys::rte_socket_id().try_into().unwrap()
        // );
        dpdk_sys::rte_eal_mp_wait_lcore();
        dpdk_sys::rte_eal_cleanup();
    }
}
