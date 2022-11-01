use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::ptr::null_mut;
use std::thread;
use std::sync::Arc;
use std::io::Error;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ffi::c_void;

use crate::config::*;
use crate::core::memory::array;
use crate::core::thread::thread::spawn;
use crate::parser::parser;
use crate::parser::header;
use crate::worker::rx;


struct DataPlaneDB<'a> {
    headers: array::Array<header::Header>,
    interface_db_list: array::Array<InterfaceDB<'a>>,
    // parsed_hdr_list: array::Array<array::Array<(&'a header::Header, usize)>>,
    // parser: parser::Parser,
}

struct InterfaceDB<'a> {
    name: String,
    parser: parser::Parser<'a>,
}

struct CacheDB {

}

struct TableDB {

} 


fn init_dataplane_db(switch_config: &SwitchConfig) -> DataPlaneDB {
    let dp_config = &switch_config.dataplane;

    // gen hdr_list
    let hdr_confs = &dp_config.headers;
    let mut headers: array::Array<header::Header> = array::Array::<header::Header>::new(hdr_confs.len());
    for (i, hdr_conf) in hdr_confs.iter().enumerate() {
        headers.write(i, header::Header::new(&hdr_conf.fields, &hdr_conf.used_fields));
    }

    let mut interface_db_list = array::Array::<InterfaceDB>::new(switch_config.interface_configs.len());
    for (i, interface_conf) in switch_config.interface_configs.iter().enumerate() {
        interface_db_list.write(i, InterfaceDB {
            name: (&interface_conf.if_name).to_string(),
            parser: parser::Parser::new(&switch_config.parser_wasm, 512, hdr_confs.len()),
        })
    }

    DataPlaneDB {
        headers,
        interface_db_list,
    }
}


fn start_workers(dp_db: &DataPlaneDB) {
    for i in 0..dp_db.interface_db_list.len() {
        let mut rx_args = rx::RxArgs {
            name: dp_db.interface_db_list[i].name.to_string(),
            parser: &dp_db.interface_db_list[i].parser,
        };
        spawn(rx::start_rx, &mut rx_args as *mut rx::RxArgs as *mut c_void);
    }
}


// CP to DP TCP Stream
// fn cp_stream_handler(mut stream: TcpStream, dp_db_arc: Arc<DataPlaneDB>) -> Result<(), Error> {
// fn cp_stream_handler(mut stream: TcpStream, dp_db_arc: *mut DataPlaneDB) -> Result<(), Error> {
fn cp_stream_handler(mut stream: TcpStream) -> Result<(), Error> {
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


// main core
pub fn start_controller(switch_config: &SwitchConfig) {
    println!("init dataplane db");
    let dp_db = init_dataplane_db(switch_config);

    println!("start workers");
    start_workers(&dp_db);

    // let dp_db_arc = Arc::new(dp_db);
    // let dp_db_ptr = &mut dp_db as *mut DataPlaneDB;

    println!("🚀Launch DP Server  {}", switch_config.listen_address);
    let listener = TcpListener::bind(&switch_config.listen_address).expect("failed to start dp server");
    listener.set_nonblocking(true).unwrap();
    loop {
        // connectio check
        match listener.accept() {
            Ok((client, addr)) => {
                // let dp_db_arc_clone = dp_db_arc.clone();
                thread::spawn(move || {
                    // cp_stream_handler(client, dp_db_arc_clone);
                    // cp_stream_handler(client, dp_db_ptr);
                    cp_stream_handler(client);
                });
            },
            _ => {},
        }
    }





    /* ------------------------
     * Header
     * --------------------------*/
    // println!("😟Loading Header");
    // let dp_config_json = get_sample_dp_config_header();
    // let dp_config: DpConfig = serde_json::from_str(&dp_config_json).unwrap();

    // let hdr_def_num = dp_config.headers.len();
    // println!("{}", hdr_def_num);
    // let hdr_defs = dpdk_memory::malloc::<header::Header>("header_definitions".to_string(), hdr_def_num as u32);

    // for (i, hdr_conf) in dp_config.headers.iter().enumerate() {
    //     unsafe {
    //         *hdr_defs.offset(i as isize) = header::Header::new((&*hdr_conf.0).to_string(), &hdr_conf.1.fields, &hdr_conf.1.used_fields);
    //     }
    // }
    // println!("👍Done");


    // /* ------------------------
    //  * Parser
    //  * --------------------------*/
    // println!("😟Loading Parser");
    // let mut f = File::open(&switch_config.parser_path).unwrap();
    // let metadata = std::fs::metadata(&switch_config.parser_path).unwrap();
    // let mut parser_bin = vec![0;metadata.len() as usize];
    // f.read(&mut parser_bin).unwrap();

    // let llvm_compiler = LLVM::default();
    // let parser_store = wasmer::Store::new(&wasmer::Universal::new(llvm_compiler).engine());
    // // let parser_store = wasmer::Store::default();
    // let parser_module = wasmer::Module::from_binary(&parser_store, &parser_bin).unwrap();

    // let parser_fn_pkt_read = wasmer::Function::new_native(&parser_store, parser::wasm_pkt_read);
    // let parser_fn_extract_header = wasmer::Function::new_native(&parser_store, parser::wasm_extract_header);
    // let parser_linear_memory = wasmer::Memory::new(&parser_store, wasmer::MemoryType::new(1, None, false)).unwrap();
    // let parser_import_object = wasmer::imports! {
    //     "env" => {
    //         "pkt_read" => parser_fn_pkt_read,
    //         "extract_header" => parser_fn_extract_header,
    //         "__linear_memory" => parser_linear_memory,
    //     },
    // };

    // let parser_instance = wasmer::Instance::new(&parser_module, &parser_import_object).unwrap();
    // let fn_parse = parser_instance.exports.get_function("parse").unwrap();
    // println!("👍Done");




    // /* ------------------------
    //  * Table
    //  * --------------------------*/





    // /* ------------------------
    //  * Pipeline
    //  * --------------------------*/
    // let pipelines = dpdk_memory::malloc::<wasmer::Function>("pipelines".to_string(), 100);
    

    // /* ------------------------
    //  * cache (L1, L3)
    //  * --------------------------*/
    // let l1_cache_len = 65535;
    // let l1_cache = dpdk_memory::malloc::<u8>("l1_cache".to_string(), l1_cache_len);
    // let l1_key_max_len = 64;
    // let l1_cache_key = dpdk_memory::malloc::<u8>("l1_key".to_string(), l1_cache_len * l1_key_max_len);



    // /* ------------------------
    //  * start interface
    //  * --------------------------*/
    // for config in switch_config.interface_configs {
    //     start_interface(config);
    // }





    // /* ------------------------
    //  * rx core (load balancer)
    //  * --------------------------*/

    // let lb_filter_len = 65535;
    // let lb_filter = dpdk_memory::malloc::<u8>("lb_filter".to_string(), lb_filter_len);

    // // let fib_core_ring_size = 4096;
    // let fib_core_ring_size = 65536;
    // let mut fib_core_rings = Vec::new();
    // fib_core_rings.push(dpdk_memory::Ring::new("fib1", fib_core_ring_size));
    // fib_core_rings.push(dpdk_memory::Ring::new("fib2", fib_core_ring_size));

    // let mut rx_start_args = rx::RxStartArgs {
    //     if_name: &switch_config.if_name,
    //     hdrs: hdr_defs,
    //     hdrs_len: hdr_def_num as u32,
    //     parser: fn_parse,
    //     l1_cache,
    //     l1_cache_key,
    //     l1_key_max_len,
    //     lb_filter,
    //     fib_core_rings: &fib_core_rings,
    // };


    // /* ------------------------
    //  * cache core
    //  * --------------------------*/
    // let mut fib_start_args_list = Vec::new();
    // fib_start_args_list.push(fib::FibStartArgs {
    //     fib_core_ring: &fib_core_rings[0],
    //     core_id: 0,
    // });
    // fib_start_args_list.push(fib::FibStartArgs {
    //     fib_core_ring: &fib_core_rings[1],
    //     core_id: 1,
    // });


    // // run switch dp
    // allocate_core_to_worker(switch_config, &mut rx_start_args, &mut fib_start_args_list);


    // let server_address = switch_config.listen_address;
}
