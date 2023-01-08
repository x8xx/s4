mod dpdk;
mod mode;
// mod tx;


use std::io::stdin;
use std::io::stdout;
use std::io::Write;
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


    println!("e_pktgen shell");
    let mut cmd_str = String::new();
    loop {
        dpdk::thread::thread_init();

        cmd_str = "".to_string();
        print!(">> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut cmd_str).unwrap();
        cmd_str = cmd_str.trim_right().to_owned();
        let mut cmd_spw = cmd_str.split_whitespace();
        let mode = cmd_spw.next();
        let arg = cmd_spw.next();

        if mode.is_none() {
            continue;
        }
        let mode = mode.unwrap();

        if mode == "exit" {
            break;
        }
        

        if mode == "t" {
            if arg.is_none() {
                continue;
            }
            let arg = arg.unwrap();
            let execution_time: u64 = arg.parse().unwrap();
            let start_locker = dpdk::memory::Locker::new();
            let end_locker = dpdk::memory::Locker::new();

            // rx
            let mut rx_args_list = Vec::new();
            for i in 0..max_rx_queues {
                rx_args_list.push(mode::time::rx::RxArgs {
                    port_number,
                    queue: i,
                    execution_time,
                    start_locker,
                    end_locker,

                });
            }

            for args in rx_args_list.iter_mut() {
                if !dpdk::thread::spawn(mode::time::rx::start_rx, args as *const mode::time::rx::RxArgs as *mut c_void) {
                    dpdk::common::cleanup();
                    panic!("faild start threadrx");
                }
            }


            // gen
            let mut gen_args_list = Vec::new();
            for i in 0..max_tx_queues - 1 {
                gen_args_list.push(mode::time::gen::GenArgs {
                    batch_count: 32,
                    gen_lib_path: gen_lib_path.to_string(),
                    interface: dpdk::interface::Interface {
                        port_number,
                        queue_number: i as u16,
                    },
                    start_locker: None,
                    end_locker: dpdk::memory::Locker::new(),
                });
            }
            gen_args_list.push(mode::time::gen::GenArgs {
                batch_count: 32,
                gen_lib_path: gen_lib_path.to_string(),
                interface: dpdk::interface::Interface {
                    port_number,
                    queue_number: (max_tx_queues - 1)as u16,
                },
                start_locker: Some(start_locker),
                end_locker: dpdk::memory::Locker::new(),
            });

            for args in gen_args_list.iter_mut() {
                if !dpdk::thread::spawn(mode::time::gen::start_gen, args as *mut mode::time::gen::GenArgs as *mut c_void) {
                    dpdk::common::cleanup();
                    panic!("faild start thread gen");
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            end_locker.wait();
            for args in gen_args_list.iter_mut() {
                args.end_locker.unlock();
            }
            dpdk::thread::thread_wait();

            start_locker.free();
            end_locker.free();
            for args in gen_args_list.iter_mut() {
                args.end_locker.free();
            }
        } else if mode == "c" {
            if arg.is_none() {
                continue;
            }
            let arg = arg.unwrap();
            let execution_time: u64 = arg.parse().unwrap();

            let arg = cmd_spw.next();
            if arg.is_none() {
                continue;
            }
            let arg = arg.unwrap();
            let gen_count: u64 = arg.parse().unwrap();

            let mut gen_args = mode::count::gen::GenArgs {
                batch_count: 32,
                gen_count,
                gen_lib_path: gen_lib_path.to_string(),
                interface: dpdk::interface::Interface {
                    port_number,
                    queue_number: 0 as u16,
                },
            };

            if !dpdk::thread::spawn(mode::count::gen::start_gen, &mut gen_args as *mut mode::count::gen::GenArgs as *mut c_void) {
                dpdk::common::cleanup();
                panic!("faild start thread gen");
            }


            let mut rx_args = mode::count::rx::RxArgs {
                port_number,
                queue: 0,
                execution_time,
            };

            if !dpdk::thread::spawn(mode::count::rx::start_rx, &mut rx_args as *mut mode::count::rx::RxArgs as *mut c_void) {
                dpdk::common::cleanup();
                panic!("faild start threadrx");
            }
            dpdk::thread::thread_wait();
        }
    }

    dpdk::common::cleanup();
}
