use std::io::prelude::*;
use std::thread;
// use std::sync::Arc;
use std::io::Error;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ffi::c_void;

use crate::config::*;
use crate::core::memory::array;
use crate::core::thread::thread::spawn;
use crate::parser::{header, parser};
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::table::Table;
use crate::worker;


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


struct WorkerArgs<'a> {
    rx_args_list: array::Array<worker::rx::RxArgs<'a>>,
    pipeline_args_list: array::Array<worker::pipeline::PipelineArgs<'a>>,
}

fn start_workers(worker_args: &mut WorkerArgs) {
    for i in 0..worker_args.rx_args_list.len() {
        spawn(worker::rx::start_rx, &mut worker_args.rx_args_list[i] as *mut worker::rx::RxArgs as *mut c_void);
    }

    for i in 0..worker_args.pipeline_args_list.len() {
        spawn(worker::pipeline::start_pipeline, &mut worker_args.pipeline_args_list[i] as *mut worker::pipeline::PipelineArgs as *mut c_void);
    }
}


// main core
pub fn start_controller(switch_config: &SwitchConfig) {
    println!("init dataplane db");
    let dp_config = &switch_config.dataplane;

    // header_list
    let hdr_confs = &dp_config.headers;
    let mut header_list: array::Array<header::Header> = array::Array::<header::Header>::new(hdr_confs.len());
    for (i, hdr_conf) in hdr_confs.iter().enumerate() {
        header_list.init(i, header::Header::new(&hdr_conf.fields, &hdr_conf.used_fields));
    }

    //table_list
    let table_confs = &dp_config.tables;
    let mut table_list = array::Array::<Table>::new(table_confs.len());
    for (i, table_conf) in table_confs.iter().enumerate() {
        table_list.init(i, Table::new(table_conf, &header_list));
    }

    // rx_args_list
    let mut rx_args_list = array::Array::<worker::rx::RxArgs>::new(switch_config.interface_configs.len());
    for (i, interface_conf) in switch_config.interface_configs.iter().enumerate() {
        rx_args_list.init(i, worker::rx::RxArgs {
            name: (&interface_conf.if_name).to_string(),
            parser: parser::Parser::new(&switch_config.parser_wasm, 512, hdr_confs.len()),
        })
    }

    // pipeline_args_list
    let mut pipeline_args_list = array::Array::<worker::pipeline::PipelineArgs>::new(switch_config.pipeline_core_num as usize);
    for i in 0..pipeline_args_list.len() {
        pipeline_args_list.init(i, worker::pipeline::PipelineArgs {
            pipeline: Pipeline::new(&switch_config.pipeline_wasm, &table_list),
        });
    }

    println!("start workers");
    let mut worker_args = WorkerArgs {
        rx_args_list,
        pipeline_args_list,
    };
    start_workers(&mut worker_args);



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
}
