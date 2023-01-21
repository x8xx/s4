mod dpdk;
mod cmd;
mod mode;


use std::env;
// use std::fs;
// use std::ffi::c_void;
use getopts::Options;
// use yaml_rust::YamlLoader;


fn main() {
    let args_start_index = dpdk::common::init();
    let mut args: Vec<String> = env::args().collect();
    let mut args: Vec<String> = args[args_start_index as usize..].to_vec();
    let shell_args_start_index = args.iter().position(|arg| arg == "--");
    let shell_args = if shell_args_start_index.is_none() {
        None
    } else {
        if args.len() - 1 ==  shell_args_start_index .unwrap(){
            None
        } else {
            Some(args.split_off(shell_args_start_index.unwrap()).split_off(1))
        }
    };


    // let pktgen_args: &[String] = &args[args_start_index as usize..];

    // let config_path = &pktgen_args[0];
    // let config = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap();

    // get config
    // let general_config = &config[0]["general"];
    // let interface_name = general_config["interface"].clone().into_string().unwrap();
    // let tap_name = general_config["tap"].clone().into_string().unwrap();
    // let execution_time = general_config["execution_time"].as_i64().unwrap() as u64;
    // println!("interface_name: {}", interface_name);
    // println!("tap_name: {}", tap_name);
    // println!("execution_time: {}", execution_time);

    let mut opts = Options::new();
    opts.optopt("i", "interface", "interface name", "");
    opts.optopt("m", "mode", "boot mode", "");

    let matches = opts.parse(args).unwrap();
    let interface: String = matches.opt_get::<String>("i").unwrap().unwrap();
    let mode: String = matches.opt_get::<String>("m").unwrap().unwrap();

    // let gen_lib_path = &pktgen_args[1];
    let gen_lib_path = "test";



    if mode == "shell" {
        mode::shell::main(interface, shell_args);
    } else {
        mode::interactive::main(interface);
    }


    dpdk::common::cleanup();
}
