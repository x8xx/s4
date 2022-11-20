use std::io::prelude::*;
use std::thread;
use std::sync::RwLock;
use std::io::Error;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ffi::c_void;

use crate::config::*;
use crate::core::network::interface::Interface;
use crate::core::memory::array::Array;
use crate::core::memory::ring;
use crate::core::thread::thread::spawn;
use crate::parser::header;
use crate::parser::parser;
use crate::cache::cache::CacheElement;
use crate::cache::tss::TupleSpace;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::table::Table;
use crate::worker;


/**
 * DataPlane to ControlPlane
 */
fn cp_stream_handler(mut stream: TcpStream, table_list: Array<RwLock<Table>>) -> Result<(), Error> {
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


/**
 *
 */
fn create_new_cache(ring: ring::Ring) {
    loop {

    }
}


/**
 * start worker
 */
struct WorkerArgs {
    rx_args_list: Array<worker::rx::RxArgs>,
    pipeline_args_list: Array<worker::pipeline::PipelineArgs>,
    tx_args_list: Array<worker::tx::TxArgs>,
}

fn start_workers(worker_args: &mut WorkerArgs) {
    for i in 0..worker_args.rx_args_list.len() {
        spawn(worker::rx::start_rx, &mut worker_args.rx_args_list[i] as *mut worker::rx::RxArgs as *mut c_void);
    }

    for i in 0..worker_args.pipeline_args_list.len() {
        spawn(worker::pipeline::start_pipeline, &mut worker_args.pipeline_args_list[i] as *mut worker::pipeline::PipelineArgs as *mut c_void);
    }

    for i in 0..worker_args.tx_args_list.len() {
        spawn(worker::tx::start_tx, &mut worker_args.tx_args_list[i] as *mut worker::tx::TxArgs as *mut c_void);
    }
}


/**
 * launch dplane controller (main core)
 */
pub fn start_controller(switch_config: &SwitchConfig) {
    println!("init dataplane db");
    let dp_config = &switch_config.dataplane;

    // header_list
    let hdr_confs = &dp_config.headers;
    let mut header_list: Array<header::Header> = Array::<header::Header>::new(hdr_confs.len());
    for (i, hdr_conf) in hdr_confs.iter().enumerate() {
        header_list.init(i, header::Header::new(&hdr_conf.fields, &hdr_conf.used_fields, &hdr_conf.parse_fields));
    }


    //table_list
    let table_confs = &dp_config.tables;
    // let table_list = Arc::new(Array::<RwLock<Table>>::new(table_confs.len()));
    let mut table_list = Array::<RwLock<Table>>::new(table_confs.len());
    for (i, table_conf) in table_confs.iter().enumerate() {
        table_list.init(i, RwLock::new(Table::new(table_conf, header_list.clone())));
    }


    let interface_configs_len = switch_config.interface_configs.len();

    // cache 
    let mut l1_cache_list = Array::<Array<CacheElement>>::new(interface_configs_len);
    let mut lbf_list = Array::<Array<u64>>::new(interface_configs_len);
    let mut l2_cache_list = Array::<Array<Array<CacheElement>>>::new(interface_configs_len);
    let mut l3_cache = TupleSpace::new(100000);


    // to main core ring 
    let cache_creater_ring = ring::Ring::new(1024);


    let rx_batch_count = 32;
    let cache_batch_count = 32;
    let pipeline_batch_count = 32;
    let tx_batch_count = 32;


    let mut tx_ring_list = Array::<ring::Ring>::new(switch_config.interface_configs.len());
    for i in 0..switch_config.interface_configs.len() {
        tx_ring_list.init(i, ring::Ring::new(1024));
    }


    // to pipeline ring list
    // pipeline_args_list
    let mut pipeline_ring_list = Array::<ring::Ring>::new(switch_config.pipeline_core_num as usize);
    let mut pipeline_args_list = Array::<worker::pipeline::PipelineArgs>::new(switch_config.pipeline_core_num as usize);
    for i in 0..pipeline_args_list.len() {
        pipeline_ring_list.init(i, ring::Ring::new(1024));
        pipeline_args_list.init(i, worker::pipeline::PipelineArgs {
            pipeline: Pipeline::new(&switch_config.pipeline_wasm, table_list.clone()),
            ring: pipeline_ring_list[i].clone(),
            batch_count: pipeline_batch_count,
            table_list_len: table_list.len(),
            tx_ring_list: tx_ring_list.clone(),
            cache_creater_ring: cache_creater_ring.clone(),
        });
    }


    // rx_args_list
    // tx_args_list
    let mut rx_args_list = Array::<worker::rx::RxArgs>::new(switch_config.interface_configs.len());
    let mut cache_args_list = Array::<worker::cache::CacheArgs>::new(switch_config.cache_core_num as usize);
    let mut tx_args_list = Array::<worker::tx::TxArgs>::new(switch_config.interface_configs.len());
    for (i, interface_conf) in switch_config.interface_configs.iter().enumerate() {
        let mut cache_ring_list = Array::<ring::Ring>::new(switch_config.cache_core_num as usize);
        l2_cache_list.init(i, Array::new(interface_conf.cache_core_num as usize));
        for j in 0..interface_conf.cache_core_num as usize {
            cache_ring_list.init(j, ring::Ring::new(1024));
            l2_cache_list[i].init(j, Array::new(switch_config.l2_cache_size));
            cache_args_list.init(j, worker::cache::CacheArgs {
                id: j,
                ring: cache_ring_list[j].clone(),
                batch_count: cache_batch_count,
                buf_len: 512,
                l2_cache: l2_cache_list[i][j].clone(),
                l3_cache: &l3_cache,
                pipeline_ring_list: pipeline_ring_list.clone(),
            });
        }

        let interface = Interface::new(&interface_conf.if_name);
        l1_cache_list.init(i, Array::new(switch_config.l1_cache_size));
        lbf_list.init(i, Array::new(switch_config.l2_cache_size));
        rx_args_list.init(i, worker::rx::RxArgs {
            id: i,
            interface: interface.clone(),
            parser: parser::Parser::new(&switch_config.parser_wasm),
            batch_count: rx_batch_count,
            pktbuf_len: 512,
            l1_hash_seed: 417,
            l2_hash_seed: 417,
            l1_cache: l1_cache_list[i].clone(),
            lbf:  lbf_list[i].clone(),
            l2_key_max_len: 30,
            header_list: header_list.clone(),
            cache_ring_list,
            pipeline_ring_list: pipeline_ring_list.clone(),
        });


        tx_args_list.init(i, worker::tx::TxArgs {
            interface: interface.clone(),
            ring: tx_ring_list[i].clone(),
            batch_count: tx_batch_count,
        });
    }


    println!("start workers");
    let mut worker_args = WorkerArgs {
        rx_args_list,
        pipeline_args_list,
        tx_args_list,
    };
    start_workers(&mut worker_args);


    // run cache crater thread
    let cache_creater_ring_for_main_core = cache_creater_ring.clone();
    thread::spawn(move || {
        create_new_cache(cache_creater_ring_for_main_core);
    });


    println!("🚀Launch DP Server  {}", switch_config.listen_address);
    let listener = TcpListener::bind(&switch_config.listen_address).expect("failed to start dp server");
    listener.set_nonblocking(true).unwrap();
    loop {
        match listener.accept() {
            Ok((client, addr)) => {
                // let table_list_clone = Arc::clone(&table_list);
                let table_list_clone = table_list.clone();
                thread::spawn(move || {
                    cp_stream_handler(client, table_list_clone);
                });
            },
            _ => {},
        }
    }
}
