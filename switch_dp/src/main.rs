mod dpdk;
mod controller;
mod worker;
mod fib;
mod config;
use std::env;


fn main() {
    let switch_args_start_index = dpdk::dpdk::init();
    let args: Vec<String> = env::args().collect();
    let switch_args: &[String] = &args[switch_args_start_index as usize..];
    let switch_config = config::parse_switch_args(switch_args);

    // controller start (main core)
    controller::controller_start(&switch_config);

    dpdk::dpdk::cleanup();
}
