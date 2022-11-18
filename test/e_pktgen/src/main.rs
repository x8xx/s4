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

    let interface = dpdk::interface::Interface::new(&name);


    // gen::gen(&mut device, &config[0], methods);

    // let mut methods: HashMap<&str, FnMethod> = HashMap::new();
    // methods.insert("tcp", method::tcp::gen_tcp_packet);
    // run::run(methods);

    dpdk::common::cleanup();
}
