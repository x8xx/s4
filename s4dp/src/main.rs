mod core;
mod config;
mod controller;
mod parser;
// mod cache;
// mod pipeline;
mod worker;

use std::env;

fn main() {
    #[cfg(feature="dpdk")]
    let switch_args_start_index = core::helper::dpdk::init();

    let args: Vec<String> = env::args().collect();
    let switch_args: &[String] = &args[switch_args_start_index as usize..];
    let switch_config = config::parse_switch_args(switch_args);
    
    controller::start_controller(&switch_config);

    #[cfg(feature="dpdk")]
    core::helper::dpdk::cleanup();
}
