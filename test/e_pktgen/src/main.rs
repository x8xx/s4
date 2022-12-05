mod method;
mod header;
mod gen;
mod dpdk;
mod tx;
mod rx;


use std::env;
use std::fs;
use std::ffi::c_void;
use yaml_rust::YamlLoader;


fn main() {
    let args_start_index = dpdk::common::init();
    let args: Vec<String> = env::args().collect();
    let pktgen_args: &[String] = &args[args_start_index as usize..];

    let config_path = &pktgen_args[0];
    let config = YamlLoader::load_from_str(&fs::read_to_string(config_path).unwrap().to_string()).unwrap();

    // get config
    let general_config = &config[0]["general"];
    let interface_name = general_config["interface"].clone().into_string().unwrap();
    let tap_name = general_config["tap"].clone().into_string().unwrap();
    let execution_time = general_config["execution_time"].as_i64().unwrap() as u64;
    println!("interface_name: {}", interface_name);
    println!("tap_name: {}", tap_name);
    println!("execution_time: {}", execution_time);

    let gen_lib_path = &pktgen_args[1];

    let interface = dpdk::interface::Interface::new(&interface_name);

    let rx_args = rx::RxArgs {
        interface: interface.clone(),
        execution_time,
    };

    let tx_ring = dpdk::memory::Ring::new(8192);
    let tx_args = tx::TxArgs {
        // tap_name: tap_name.clone(),
        pkt_batch_count: 256,
        batch_count: 32,
        ring: tx_ring.clone(),
        interface: interface.clone(),
        execution_time,
        rx_args: &rx_args as *const rx::RxArgs as *mut c_void,
    };

    let gen_args = gen::GenArgs {
        tap_name: tap_name.clone(),
        batch_count: 256,
        execution_time,
        tx_ring: tx_ring.clone(),
        tx_args: &tx_args as *const tx::TxArgs as *mut c_void,
        gen_lib_path: gen_lib_path.to_string(),
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
