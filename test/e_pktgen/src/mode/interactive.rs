use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::ffi::c_void;
use crate::cmd;
use crate::dpdk;


pub fn main(interface: String) {
    let (port_number, max_rx_queues, max_tx_queues) = dpdk::interface::Interface::init(&interface);
    // let max_tx_queues = 14;
    println!("Port{}: max_rx_queues {}, max_tx_queues {}", port_number, max_rx_queues, max_tx_queues);

    let mut libpktgen = "".to_string();
    let mut cmd_str = String::new();
    loop {
        dpdk::thread::thread_init();

        cmd_str = "".to_string();
        print!(">> ");
        stdout().flush().unwrap();
        stdin().read_line(&mut cmd_str).unwrap();
        cmd_str = cmd_str.trim_right().to_owned();

        let mut cmd_spw = cmd_str.split_whitespace();
        let cmd = cmd_spw.next();
        let arg = cmd_spw.next();

        if cmd.is_none() {
            continue;
        }
        let cmd = cmd.unwrap();


        if cmd == "exit" {
            break;
        } else if cmd == "set" {
            if arg.is_none() {
                continue;
            }
            let set_var_name: String = arg.unwrap().parse().unwrap();
            if set_var_name == "libpktgen" {
                let arg = cmd_spw.next();
                if arg.is_none() {
                    continue;
                }
                libpktgen = arg.unwrap().parse().unwrap();
                continue;
            }
        }
        

        if cmd == "t" {
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
                rx_args_list.push(cmd::time::rx::RxArgs {
                    port_number,
                    queue: i,
                    execution_time,
                    start_locker,
                    end_locker,

                });
            }

            for args in rx_args_list.iter_mut() {
                if !dpdk::thread::spawn(cmd::time::rx::start_rx, args as *const cmd::time::rx::RxArgs as *mut c_void) {
                    dpdk::common::cleanup();
                    panic!("faild start threadrx");
                }
            }


            // gen
            let mut gen_args_list = Vec::new();
            for i in 0..max_tx_queues - 1 {
                gen_args_list.push(cmd::time::gen::GenArgs {
                    batch_count: 32,
                    gen_lib_path: libpktgen.to_string(),
                    interface: dpdk::interface::Interface {
                        port_number,
                        queue_number: i as u16,
                    },
                    start_locker: None,
                    end_locker: dpdk::memory::Locker::new(),
                });
            }
            gen_args_list.push(cmd::time::gen::GenArgs {
                batch_count: 32,
                gen_lib_path: libpktgen.to_string(),
                interface: dpdk::interface::Interface {
                    port_number,
                    queue_number: (max_tx_queues - 1)as u16,
                },
                start_locker: Some(start_locker),
                end_locker: dpdk::memory::Locker::new(),
            });

            for args in gen_args_list.iter_mut() {
                if !dpdk::thread::spawn(cmd::time::gen::start_gen, args as *mut cmd::time::gen::GenArgs as *mut c_void) {
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
        } else if cmd == "c" {
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

            let mut gen_args = cmd::count::gen::GenArgs {
                batch_count: 32,
                gen_count,
                gen_lib_path: libpktgen.to_string(),
                interface: dpdk::interface::Interface {
                    port_number,
                    queue_number: 0 as u16,
                },
            };

            if !dpdk::thread::spawn(cmd::count::gen::start_gen, &mut gen_args as *mut cmd::count::gen::GenArgs as *mut c_void) {
                dpdk::common::cleanup();
                panic!("faild start thread gen");
            }


            let mut rx_args = cmd::count::rx::RxArgs {
                port_number,
                queue: 0,
                execution_time,
            };

            if !dpdk::thread::spawn(cmd::count::rx::start_rx, &mut rx_args as *mut cmd::count::rx::RxArgs as *mut c_void) {
                dpdk::common::cleanup();
                panic!("faild start threadrx");
            }
            dpdk::thread::thread_wait();
        }
    }

}
