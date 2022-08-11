mod dpdk;
mod controller;
mod worker;
mod fib;
mod memory;
mod config;
use std::env;
use std::os::raw::c_char;
use std::ffi::CString;


fn main() {
    // init
    // let cargs: Vec<_> = env::args().map(|s| CString::new(s).unwrap()).collect();
    // let mut dpdk_cargs: Vec<_> = cargs.iter().map(|s| s.as_ptr() as *mut c_char).collect();
    // let switch_args_start_index = unsafe {
    //     let ret = dpdk_sys::rte_eal_init(dpdk_cargs.len() as i32, dpdk_cargs.as_mut_ptr());
    //     if ret < 0 {
    //         panic!("Cannot init EAL\n");
    //     }

    //     (ret + 1) as usize
    // };


    // dpdk::dpdk::init();
    let switch_args_start_index = dpdk::dpdk::init();

    // let args: Vec<&str> = cargs.iter().map(|cs| cs.to_str().unwrap()).collect();
    let args: Vec<String> = env::args().collect();
    let switch_args: &[String] = &args[switch_args_start_index as usize..];
    let switch_config = config::parse_switch_args(switch_args);


    // load shared lib
    // unsafe {
    //     dpdk_sys::output_test_log();
    //     dpdk_sys::load_rte_eth_tap();
    // }


    // port check
    let avail_port_num = unsafe { dpdk_sys::rte_eth_dev_count_avail() };
    if avail_port_num <= 0 {
        panic!("Cannot avail device\n");
    }
    println!("port {}", avail_port_num);



    // controller start (main core)
    controller::controller_start(&switch_config);


    // let rx_start_args = worker::rx::RxStartArgs {};
    // let fib_start_args = worker::fib::FibStartArgs {};

    // let worker_args = worker::WorkerArgs{
    //     rx_start_args,
    //     fib_start_args,
    // };
    // if !worker::remote_launch_worker(&switch_config, worker_args) {
    //     panic!("Failed launch worker\n");
    // }
    // println!("main_start");

    dpdk::dpdk::cleanup();
    // unsafe {
    //     dpdk_sys::rte_eal_mp_wait_lcore();
    //     dpdk_sys::rte_eal_cleanup();
    // }
}
