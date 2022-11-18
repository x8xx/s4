// mod run;
mod method;
mod header;
// mod device;
mod gen;
mod dpdk;
mod tx;
mod rx;


use std::env;
use std::fs;
use std::ffi::c_void;
use std::collections::HashMap;
use yaml_rust::YamlLoader;
use crate::method::FnMethod;


fn main() {
    let args_start_index = dpdk::common::init();
    let args: Vec<String> = env::args().collect();
    let pktgen_args: &[String] = &args[args_start_index as usize..];

    let config_path = &pktgen_args[0];
    let config = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap();

    // get config
    let general_config = &config[0]["general"];
    let name = general_config["interface"].clone().into_string().unwrap();
    println!("interface_name: {}", name);

    let interface = dpdk::interface::Interface::new(&name);

    let gen_args = gen::GenArgs {

    };

    if !dpdk::thread::spawn(gen::start_gen, &gen_args as *const gen::GenArgs as *mut c_void) {
        dpdk::common::cleanup();
        panic!("faild start thread gen");
    }


    // let mut methods: HashMap<&str, FnMethod> = HashMap::new();
    // methods.insert("tcp", method::tcp::gen_tcp_packet);
    // run::run(methods);

    
    dpdk::thread::thread_wait();
    dpdk::common::cleanup();
}
