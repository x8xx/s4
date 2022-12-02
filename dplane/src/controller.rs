mod cmd;
mod cp_stream;
mod table_controller;
mod cache_controller;

use std::thread;
use std::sync::RwLock;
use std::net::TcpListener;
use std::ffi::c_void;
use crate::config::*;
use crate::core::network::interface::Interface;
use crate::core::memory::heap::Heap;
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
 * launch dplane controller (main core)
 */
pub fn start_controller(switch_config: &SwitchConfig) {
    println!("init dataplane db");
    let dp_config = &switch_config.dataplane;

    // header_list
    let hdr_confs = &dp_config.headers;
    let mut header_max_size = dp_config.header_max_size;
    let mut header_list: Array<header::Header> = Array::<header::Header>::new(hdr_confs.len());
    for (i, hdr_conf) in hdr_confs.iter().enumerate() {
        header_list.init(i, header::Header::new(&hdr_conf.fields, &hdr_conf.used_fields, &hdr_conf.parse_fields));
    }


    //table_list
    let table_confs = &dp_config.tables;
    let mut table_list = Array::<RwLock<Table>>::new(table_confs.len());
    for (i, table_conf) in table_confs.iter().enumerate() {
        table_list.init(i, RwLock::new(Table::new(table_conf, header_list.clone())));
    }

    // initial flowentry
    // entry_count(4byte) | (table_id(1byte) | entry_buf_size(2byte) | entry)*
    let initial_table_data = &switch_config.initial_table_data;
    if initial_table_data.len() != 0 {
        let entry_count: u32 = ((initial_table_data[3] as u32) << 24) + ((initial_table_data[2] as u32) << 16) + ((initial_table_data[1] as u32) << 8) + initial_table_data[0] as u32;
        let mut pos = 4;
        for _ in 0..entry_count as usize {
            let table_id = initial_table_data[pos];
            pos += 1;
            let entry_buf_size = ((initial_table_data[pos + 1] as u16) << 8) + initial_table_data[pos] as u16;
            pos += 2;
            table_controller::add_flow_entry(&mut table_list[table_id as usize], &initial_table_data[pos..pos+entry_buf_size as usize]);
            pos += entry_buf_size as usize;
        }
    }


    let interface_configs_len = switch_config.interface_configs.len();

    // cache 
    let mut l1_cache_list = Array::<Array<RwLock<CacheElement>>>::new(interface_configs_len);
    let mut lbf_list = Array::<Array<u64>>::new(interface_configs_len);
    let mut l2_cache_list = Array::<Array<Array<RwLock<CacheElement>>>>::new(interface_configs_len);
    // let mut l3_cache_list = Array::<TupleSpace>::new(interface_configs_len);
    let mut l3_cache = TupleSpace::new(10000);


    // to main core ring 
    let cache_creater_ring = ring::Ring::new(1024);


    let rx_batch_count = 64;
    let cache_batch_count = 64;
    let pipeline_batch_count = 64;
    let tx_batch_count = 64;

    let rx_buf_size = 8192;
    let cache_buf_size = 8192;
    let pipeline_buf_size = 8192;
    let tx_buf_size = 8192;


    let mut tx_ring_list = Array::<ring::Ring>::new(switch_config.interface_configs.len() + 1);
    for i in 0..tx_ring_list.len() {
        tx_ring_list.init(i, ring::Ring::new(tx_buf_size));
    }


    // to pipeline ring list
    // pipeline_args_list
    let mut pipeline_ring_from_rx_list = Array::<ring::Ring>::new(switch_config.pipeline_core_num as usize);
    let mut pipeline_ring_from_cache_list = Array::<ring::Ring>::new(switch_config.pipeline_core_num as usize);
    let mut pipeline_args_list = Array::<worker::pipeline::PipelineArgs>::new(switch_config.pipeline_core_num as usize);

    for i in 0..pipeline_args_list.len() {
        pipeline_ring_from_rx_list.init(i, ring::Ring::new(pipeline_buf_size));
        pipeline_ring_from_cache_list.init(i, ring::Ring::new(pipeline_buf_size));

        pipeline_args_list.init(i, worker::pipeline::PipelineArgs {
            pipeline: Pipeline::new(&switch_config.pipeline_wasm, table_list.clone()),
            ring_from_rx: pipeline_ring_from_rx_list[i].clone(),
            ring_from_cache: pipeline_ring_from_cache_list[i].clone(),
            batch_count: pipeline_batch_count,
            table_list_len: table_list.len(),
            header_max_size,
            tx_ring_list: tx_ring_list.clone(),
            cache_creater_ring: cache_creater_ring.clone(),
        });
    }

    // 512MB
    let mut heap = Heap::new(536870912); 
    
    // rx_args_list
    // tx_args_list
    let mut rx_args_list = Array::<worker::rx::RxArgs>::new(switch_config.interface_configs.len());
    let mut cache_args_list = Array::<worker::cache::CacheArgs>::new(switch_config.cache_core_num as usize);
    let mut tx_args_list = Array::<worker::tx::TxArgs>::new(switch_config.interface_configs.len());

    let mut cache_args_count = 0;
    for (i, interface_conf) in switch_config.interface_configs.iter().enumerate() {
        let mut cache_ring_list = Array::<ring::Ring>::new(switch_config.cache_core_num as usize);
        l2_cache_list.init(i, Array::new(interface_conf.cache_core_num as usize));
        for j in 0..interface_conf.cache_core_num as usize {
            println!("init: cache core rx-{}, cache-{}", i, j);
            cache_ring_list.init(j, ring::Ring::new(1024));
            l2_cache_list[i].init(j, Array::new(switch_config.l2_cache_size));
            {
                for k in 0..l2_cache_list[i][j].len() {
                    l2_cache_list[i][j].init(k, RwLock::new(CacheElement {
                        key: heap.malloc(header_max_size),
                        key_len: 0,
                        data: heap.malloc(table_list.len()),
                    }));
                }
            }

            cache_args_list.init(cache_args_count, worker::cache::CacheArgs {
                id: j,
                ring: cache_ring_list[j].clone(),
                batch_count: cache_batch_count,
                buf_len: cache_buf_size,
                header_max_size,
                l2_cache: l2_cache_list[i][j].clone(),
                l3_cache: &l3_cache,
                pipeline_ring_list: pipeline_ring_from_cache_list.clone(),
            });
            cache_args_count += 1;
        }

        println!("init: rx core rx-{}", i);
        let interface = Interface::new(&interface_conf.if_name);
        l1_cache_list.init(i, Array::new(switch_config.l1_cache_size));
        {
            for j in 0..l1_cache_list[i].len() {
                l1_cache_list[i].init(j, RwLock::new(CacheElement {
                    key: heap.malloc(header_max_size),
                    key_len: 0,
                    data: heap.malloc(table_list.len()),
                }));
            }
        }

        lbf_list.init(i, Array::new(switch_config.l2_cache_size));
        rx_args_list.init(i, worker::rx::RxArgs {
            id: i,
            interface: interface.clone(),
            parser: parser::Parser::new(&switch_config.parser_wasm),
            batch_count: rx_batch_count,
            pktbuf_len: rx_buf_size,
            l1_hash_seed: 417,
            l2_hash_seed: 417,
            l1_cache: l1_cache_list[i].clone(),
            lbf:  lbf_list[i].clone(),
            l2_key_max_len: 30,
            header_list: header_list.clone(),
            cache_ring_list,
            pipeline_ring_list: pipeline_ring_from_rx_list.clone(),
        });


        println!("init: tx core tx-{}", i);
        tx_args_list.init(i, worker::tx::TxArgs {
            interface: interface.clone(),
            ring: tx_ring_list[i + 1].clone(),
            batch_count: tx_batch_count,
        });
    }


    // start worker
    println!("start workers");
    for i in 0..rx_args_list.len() {
        spawn(worker::rx::start_rx, &mut rx_args_list[i] as *mut worker::rx::RxArgs as *mut c_void);
    }

    for i in 0..cache_args_list.len() {
        spawn(worker::cache::start_cache, &mut cache_args_list[i] as *mut worker::cache::CacheArgs as *mut c_void);
    }

    for i in 0..pipeline_args_list.len() {
        println!("pp {}" , i);
        spawn(worker::pipeline::start_pipeline, &mut pipeline_args_list[i] as *mut worker::pipeline::PipelineArgs as *mut c_void);
    }

    for i in 0..tx_args_list.len() {
        println!("tx {}" , i);
        spawn(worker::tx::start_tx, &mut tx_args_list[i] as *mut worker::tx::TxArgs as *mut c_void);
    }


    // run cache crater thread
    let cache_creater_ring_for_main_core = cache_creater_ring.clone();
    let table_list_clone = table_list.clone();
    thread::spawn(move || {
        cache_controller::create_new_cache(cache_creater_ring_for_main_core,
                                           table_list_clone,
                                           l1_cache_list.clone(),
                                           lbf_list.clone(),
                                           l2_cache_list.clone());
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
                    cp_stream::stream_handler(client, table_list_clone);
                });
            },
            _ => {},
        }
    }
}
