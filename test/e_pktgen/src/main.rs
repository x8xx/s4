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

    let (port_number, max_rx_queues, max_tx_queues) = dpdk::interface::Interface::init(&interface_name);
    // let max_tx_queues = 14;


    println!("Port{}: max_rx_queues {}, max_tx_queues {}", port_number, max_rx_queues, max_tx_queues);


    // let tx_ring = dpdk::memory::Ring::new(8192);
    // let tx_args = tx::TxArgs {
    //     // tap_name: tap_name.clone(),
    //     pkt_batch_count: 32,
    //     batch_count: 32,
    //     ring: tx_ring.clone(),
    //     interface: interface.clone(),
    //     execution_time,
    //     rx_args: &rx_args as *const rx::RxArgs as *mut c_void,
    // };

    let mut gen_args_list = Vec::new();
    for i in 0..max_tx_queues {
        gen_args_list.push(gen::GenArgs {
            tap_name: tap_name.clone(),
            batch_count: 32,
            execution_time,
            gen_lib_path: gen_lib_path.to_string(),
            interface: dpdk::interface::Interface {
                port_number,
                queue_number: i as u16,
            },
        });
    }

//     let gen_args = gen::GenArgs {
//         tap_name: tap_name.clone(),
//         batch_count: 32,
//         execution_time,
//         // tx_ring: tx_ring.clone(),
//         // tx_args: &tx_args as *const tx::TxArgs as *mut c_void,
//         gen_lib_path: gen_lib_path.to_string(),
//     };

    for args in gen_args_list.iter_mut() {
        if !dpdk::thread::spawn(gen::start_gen, args as *mut gen::GenArgs as *mut c_void) {
            dpdk::common::cleanup();
            panic!("faild start thread gen");
        }
        // std::thread::sleep(std::time::Duration::from_secs(2));
    }


    let rx_result = Vec::<u64>::with_capacity(max_rx_queues as usize);;
    let mut rx_args_list = Vec::new();
    for i in 0..max_rx_queues {
        rx_args_list.push(rx::RxArgs {
            port_number,
            queue: i,
            execution_time,
            result: unsafe { (rx_result.as_ptr() as *mut u64).offset(i as isize) },
        });
    }

    // let rx_args = rx::RxArgs {
    //     port_number,
    //     max_rx_queues,
    //     execution_time,
    // };

    for args in rx_args_list.iter_mut() {
        if !dpdk::thread::spawn(rx::start_rx, args as *const rx::RxArgs as *mut c_void) {
            dpdk::common::cleanup();
            panic!("faild start threadrx");
        }
    }

    dpdk::thread::thread_wait();

    let mut sum = 0;
    for i in 0..rx_result.len() {
        sum += rx_result[i];
    }
    println!("receive pkt count: {}", sum);

    dpdk::common::cleanup();
}
