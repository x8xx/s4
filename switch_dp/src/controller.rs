use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use crate::config::*;
use crate::dpdk::dpdk_memory;
use crate::worker::*;
use crate::fib::header;
use crate::fib::parser;
use serde::Deserialize;
use wasmer_compiler_llvm::LLVM;

fn allocate_core_to_worker(switch_config: &SwitchConfig,
                            rx_start_args: &mut rx::RxStartArgs,
                            fib_start_args: &mut fib::FibStartArgs) -> bool {
    let mut unallocated_rx_core = switch_config.rx_cores;
    let mut unallocated_fib_core = switch_config.fib_cores;
    unsafe {
        let mut lcore_id: u32 = dpdk_sys::rte_get_next_lcore(u32::MIN, 1, 0);
        while lcore_id < dpdk_sys::RTE_MAX_LCORE {
            if unallocated_rx_core > 0 {
                if !remote_launch_rx(lcore_id, rx_start_args) {
                    panic!("Failed start rx worker");
                }
                unallocated_rx_core -= 1;
            } else if unallocated_fib_core > 0 {
                if !remote_launch_fib(lcore_id, fib_start_args) {
                    panic!("Failed start fib worker");
                }
                unallocated_fib_core -= 1;
            }
            lcore_id = dpdk_sys::rte_get_next_lcore(lcore_id, 1, 0);
        }
    }
    
    !(unallocated_rx_core > 0 || unallocated_fib_core > 0)
}



#[derive(Deserialize)]
struct DpConfig {
    headers: HashMap<String, DpConfigHeader>
}

#[derive(Deserialize)]
struct DpConfigHeader {
    fields: Vec<u16>,
    used_fields: Vec<u16>,
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


// main core
pub fn controller_start(switch_config: &SwitchConfig) {
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
    // let parser_store = wasmer::Store::new(&wasmer::Universal::new(llvm_compiler).engine());
    let parser_store = wasmer::Store::default();
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
     * Pipeline
     * --------------------------*/
    let pipelines = dpdk_memory::malloc::<wasmer::Function>("pipelines".to_string(), 100);
    


    /* ------------------------
     * rx core (load balancer)
     * --------------------------*/
    let l1_cache_len = 65535;
    let l1_cache = dpdk_memory::malloc::<u8>("l1_cache".to_string(), l1_cache_len);
    let l1_key_max_len = 48;
    let l1_cache_key = dpdk_memory::malloc::<u8>("l1_key".to_string(), l1_cache_len * l1_key_max_len);

    let lb_filter_len = 65535;
    let lb_filter = dpdk_memory::malloc::<u8>("lb_filter".to_string(), lb_filter_len);

    let mut fib_core_rings = Vec::new();
    fib_core_rings.push(dpdk_memory::Ring::new("fib1", 4096));
    fib_core_rings.push(dpdk_memory::Ring::new("fib2", 4096));

    let mut rx_start_args = rx::RxStartArgs {
        if_name: &switch_config.if_name,
        hdrs: hdr_defs,
        hdrs_len: hdr_def_num as u32,
        parser: fn_parse,
        l1_cache,
        lb_filter,
        fib_core_rings: &fib_core_rings,
    };


    /* ------------------------
     * cache core
     * --------------------------*/
    let mut fib_start_args =fib::FibStartArgs {

    };


    /* ------------------------
     * pipeline core
     * --------------------------*/



    allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args);


    loop {

    }
    // run tcp
}
