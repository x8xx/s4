use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use crate::config::*;
use crate::dpdk::dpdk_memory;
use crate::worker::*;
use crate::fib::cache::*;
use crate::fib::parser::*;
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


// main core
pub fn controller_start(switch_config: &SwitchConfig) {
    /* ------------------------
     * parser
     * --------------------------*/
    let mut f = File::open(&switch_config.parser_path).unwrap();
    let metadata = std::fs::metadata(&switch_config.parser_path).unwrap();
    let mut parser_bin = vec![0;metadata.len() as usize];
    f.read(&mut parser_bin).unwrap();


    let compiler = LLVM::default();
    let parser_store = wasmer::Store::new(&wasmer::Universal::new(compiler).engine());

    let parser_module = wasmer::Module::from_binary(&parser_store, &parser_bin).unwrap();

    let mut test_array: Vec<u8> = Vec::new();
    test_array.push(0);
    test_array.push(1);
    test_array.push(2);
    test_array.push(3);
    let test_array_ptr = test_array.as_ptr();
    fn read(ptr: i64, offset: i32) -> i32 {
        println!("ptr: {}", ptr);
        unsafe {
            (*(ptr as *const u8).offset(offset as isize)) as i32
        }
    }

    let read_fn_store = wasmer::Store::default();
    let memory_store = wasmer::Store::default();
    let read_fn = wasmer::Function::new_native(&read_fn_store, read);
    let linear_memory = wasmer::Memory::new(&memory_store, wasmer::MemoryType::new(1, None, false)).unwrap();

    let parser_import_object = wasmer::imports! {
        "env" => {
            "read" => read_fn,
            "__linear_memory" => linear_memory,
        },
    };
    // let parser_import_object = wasmer::imports! {};

    let mut parser_instance = wasmer::Instance::new(&parser_module, &parser_import_object).unwrap();

    // parser_instance.exports.insert("read2", read_fn);
    for (name, ext) in parser_instance.exports.iter() {
        println!("{}", name);
    }


    let parser_fn_parse = parser_instance.exports.get_function("parse").unwrap();
    let mut parser_args: Vec<wasmer::Value> = Vec::new();
    parser_args.push(wasmer::Value::I64(test_array_ptr as i64));
    parser_args.push(wasmer::Value::I32(10));
    let parse_result = parser_fn_parse.call(&parser_args).unwrap();
    println!("parse_result: {}", parse_result[0].unwrap_i32());

    return;

    /* ------------------------
     * rx core
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
        // parser: &parser,
        l1_cache,
        lb_filter,
        fib_core_rings: &fib_core_rings,
    };


    /* ------------------------
     * fib core
     * --------------------------*/
    let mut fib_start_args =fib::FibStartArgs {

    };

    allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args);


    // run tcp
}
