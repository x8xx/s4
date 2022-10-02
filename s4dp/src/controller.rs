use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::io::Error;
use std::net::TcpListener;
use std::net::TcpStream;

use crate::config::*;
use crate::dpdk::dpdk_memory;
use crate::worker::*;
use crate::fib::header;
use crate::fib::parser;

use serde::Deserialize;
use wasmer_compiler_llvm::LLVM;

#[derive(Deserialize)]
struct DpConfig {
    headers: HashMap<String, DpConfigHeader>,
    tables: Vec<DpConfigTable>,
}

#[derive(Deserialize)]
struct DpConfigHeader {
    fields: Vec<u16>,
    used_fields: Vec<u16>,
}

#[derive(Deserialize)]
struct DpConfigTable {

}

fn get_sample_dp_config_header() -> String {
    "
        {
            \"headers\": {
                \"ethernet\": {
                    \"fields\": [48, 48, 16],
                    \"used_fields\": [0, 1, 2]
                },
                \"ipv4\": {
                    \"fields\": [4, 4, 8, 16, 16, 3, 13, 8, 8, 16, 32, 32],
                    \"used_fields\": [9, 10, 11]
                },
                \"tcp\": {
                    \"fields\": [16, 16, 32, 32, 4, 6, 6, 16 ,16, 16],
                    \"used_fields\": [0, 1]
                },
                \"udp\": {
                    \"fields\": [16, 16, 16],
                    \"used_fields\": [0, 1]
                }
            }
        }
    ".to_string()
}


fn allocate_core_to_worker(switch_config: &SwitchConfig,
                            rx_start_args: &mut rx::RxStartArgs,
                            fib_start_args_list: &mut Vec<fib::FibStartArgs>) -> bool {
    let mut unallocated_rx_core = switch_config.rx_cores;
    let mut unallocated_fib_core = switch_config.fib_cores;
    unsafe {
        let mut fib_core_id = 0;
        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            if unallocated_rx_core > 0 {
                if !remote_launch_rx(lcore_id, rx_start_args) {
                    panic!("Failed start rx worker");
                }
                unallocated_rx_core -= 1;
            } else if unallocated_fib_core > 0 {
                if !remote_launch_fib(lcore_id, &mut fib_start_args_list[fib_core_id]) {
                    panic!("Failed start fib worker");
                }
                fib_core_id += 1;
                unallocated_fib_core -= 1;
            }
            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }
    }
    
    !(unallocated_rx_core > 0 || unallocated_fib_core > 0)
}


// CP to DP TCP Stream
fn handler(mut stream: TcpStream) -> Result<(), Error> {
    let mut buffer = [0; 1024];
    loop {
        let nbytes = stream.read(&mut buffer)?;
        if nbytes == 0 {
            return Ok(());
        }
        stream.write(&buffer[..nbytes])?;
        stream.flush()?;
    }
}


struct StartInterfaceArgs {
    l1_cache: *mut u8,
    l1_cache_key: *mut u8,
    lb: *mut u8,
    l2_cache: *mut u8,
    l2_cache_key: *mut u8,
    l3_cache: *mut u8,
}


fn start_interface(config: InterfaceConfig) -> u32  {
    0
}


// main core
pub fn start_controller(switch_config: &SwitchConfig) {
    /* ------------------------
     * Header
     * --------------------------*/
    println!("😟Loading Header");
    let dp_config_json = get_sample_dp_config_header();
    let dp_config: DpConfig = serde_json::from_str(&dp_config_json).unwrap();
    let hdr_def_num = dp_config.headers.len();
    println!("{}", hdr_def_num);
    let hdr_defs = dpdk_memory::malloc::<header::Header>("header_definitions".to_string(), hdr_def_num as u32);

    for (i, hdr_conf) in dp_config.headers.iter().enumerate() {
        unsafe {
            *hdr_defs.offset(i as isize) = header::Header::new((&*hdr_conf.0).to_string(), &hdr_conf.1.fields, &hdr_conf.1.used_fields);
        }
    }
    println!("👍Done");


    /* ------------------------
     * Parser
     * --------------------------*/
    println!("😟Loading Parser");
    let mut f = File::open(&switch_config.parser_path).unwrap();
    let metadata = std::fs::metadata(&switch_config.parser_path).unwrap();
    let mut parser_bin = vec![0;metadata.len() as usize];
    f.read(&mut parser_bin).unwrap();

    let llvm_compiler = LLVM::default();
    let parser_store = wasmer::Store::new(&wasmer::Universal::new(llvm_compiler).engine());
    // let parser_store = wasmer::Store::default();
    let parser_module = wasmer::Module::from_binary(&parser_store, &parser_bin).unwrap();

    let parser_fn_pkt_read = wasmer::Function::new_native(&parser_store, parser::wasm_pkt_read);
    let parser_fn_extract_header = wasmer::Function::new_native(&parser_store, parser::wasm_extract_header);
    let parser_linear_memory = wasmer::Memory::new(&parser_store, wasmer::MemoryType::new(1, None, false)).unwrap();
    let parser_import_object = wasmer::imports! {
        "env" => {
            "pkt_read" => parser_fn_pkt_read,
            "extract_header" => parser_fn_extract_header,
            "__linear_memory" => parser_linear_memory,
        },
    };

    let parser_instance = wasmer::Instance::new(&parser_module, &parser_import_object).unwrap();
    let fn_parse = parser_instance.exports.get_function("parse").unwrap();
    println!("👍Done");


    /* ------------------------
     * Table
     * --------------------------*/





    /* ------------------------
     * Pipeline
     * --------------------------*/
    let pipelines = dpdk_memory::malloc::<wasmer::Function>("pipelines".to_string(), 100);
    

    /* ------------------------
     * cache (L1, L3)
     * --------------------------*/
    let l1_cache_len = 65535;
    let l1_cache = dpdk_memory::malloc::<u8>("l1_cache".to_string(), l1_cache_len);
    let l1_key_max_len = 64;
    let l1_cache_key = dpdk_memory::malloc::<u8>("l1_key".to_string(), l1_cache_len * l1_key_max_len);



    /* ------------------------
     * start interface
     * --------------------------*/
    for config in switch_config.interface_configs {
        start_interface(config);
    }





    /* ------------------------
     * rx core (load balancer)
     * --------------------------*/

    let lb_filter_len = 65535;
    let lb_filter = dpdk_memory::malloc::<u8>("lb_filter".to_string(), lb_filter_len);

    // let fib_core_ring_size = 4096;
    let fib_core_ring_size = 65536;
    let mut fib_core_rings = Vec::new();
    fib_core_rings.push(dpdk_memory::Ring::new("fib1", fib_core_ring_size));
    fib_core_rings.push(dpdk_memory::Ring::new("fib2", fib_core_ring_size));

    let mut rx_start_args = rx::RxStartArgs {
        if_name: &switch_config.if_name,
        hdrs: hdr_defs,
        hdrs_len: hdr_def_num as u32,
        parser: fn_parse,
        l1_cache,
        l1_cache_key,
        l1_key_max_len,
        lb_filter,
        fib_core_rings: &fib_core_rings,
    };


    /* ------------------------
     * cache core
     * --------------------------*/
    let mut fib_start_args_list = Vec::new();
    fib_start_args_list.push(fib::FibStartArgs {
        fib_core_ring: &fib_core_rings[0],
        core_id: 0,
    });
    fib_start_args_list.push(fib::FibStartArgs {
        fib_core_ring: &fib_core_rings[1],
        core_id: 1,
    });


    // run switch dp
    allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args_list);


    let server_address = switch_config.listen_address;
    // run tcp
    println!("🚀Launch DP Server  {}", server_address);
    let listener = TcpListener::bind(server_address).expect("failed to start dp server");
    for streams in listener.incoming() {
        match streams {
            // Err(e) => {},
            Err(_) => {println!("error");},
            Ok(stream) => {
                println!("");
                println!("Start Stream {}", server_address);
                println!("");
                thread::spawn(move || {
                    handler(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}