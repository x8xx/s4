// use std::collections::HashMap;
// use crate::device::Device;
// use crate::method::FnMethod;

// const GENERAL_CONFIG_KEY: &str = "General";
// const METHOD_CONFIG_KEY: &str = "Methods";

// pub fn gen(device: &mut Device, config: &yaml_rust::Yaml, methods: HashMap<&str, FnMethod>) {
//     let general_config = &config[GENERAL_CONFIG_KEY];
//     let packets = methods["tcp"](&config[METHOD_CONFIG_KEY]["tcp"], general_config["count"].as_i64().unwrap());
//     // println!("{}", config["General"]["count"].as_i64().unwrap());
//     // println!("{}", config["Environment"]["tcp"]["count"].as_i64().unwrap());
//     // println!("{}", config.len());
//     for packet in packets.iter() {
//         // println!("{:?}", packet);
//         device.send(packet);
//     }
// }

use std::ffi::c_void;
use std::mem::transmute;
use std::time::Duration;
use std::time::Instant;
use pnet::datalink;
use pnet::datalink::NetworkInterface;
use pnet::datalink::DataLinkSender;
use pnet::datalink::Channel::Ethernet;

use crate::dpdk::common::cleanup;
use crate::dpdk::memory::RingBuf;
use crate::dpdk::memory::Array;
// use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::thread::spawn;
use crate::tx;


#[repr(C)]
pub struct GenArgs {
    pub tap_name: String,
    pub execution_time: u64,
    pub tx_args: *mut c_void,
    pub gen_lib_path: String,
}


fn get_tap_tx(name: &str) -> Box<dyn DataLinkSender> {
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter().filter(|interface: &NetworkInterface| interface.name == name).next().expect("Failed get Inteface");

    let (tx, _) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("failed create channel"),
        Err(e) => panic!("{}", e),
    };

    tx
}


pub extern "C" fn start_gen(gen_args_ptr: *mut c_void) -> i32 {
    println!("start gen thread");
    let gen_args = unsafe { &mut *transmute::<*mut c_void, *mut GenArgs>(gen_args_ptr) };

    let mut tap_tx = get_tap_tx(&gen_args.tap_name);

    let mut pkt_buf_list: Array<Array<u8>> = Array::new(4096);
    let mut pkt_buf_ptr_list: Array<*mut u8> = Array::new(4096);
    for i in 0..pkt_buf_list.len() {
        pkt_buf_list.init(i, Array::new(64));
        pkt_buf_ptr_list.init(i, pkt_buf_list[i].as_ptr());
    }

    // launch tx thread
    if !spawn(tx::start_tx, gen_args.tx_args) {
        cleanup();
        panic!("faild start thread tx");
    }

    // let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(gen_args.execution_time + 10);
    let libpktgen = unsafe { libloading::Library::new(&gen_args.gen_lib_path).unwrap() };
    let fn_pktgen = unsafe { libpktgen.get::<libloading::Symbol<unsafe extern fn(buf_list: *mut *mut u8) -> usize>>(b"pktgen").unwrap() };
    loop {
        let gen_count = unsafe { fn_pktgen(pkt_buf_ptr_list.as_ptr()) };

        for i in 0..gen_count {
            tap_tx.send_to(pkt_buf_list[i].as_slice(), None);
        }

        if end_time < Instant::now() {
            return 0;
        }

        if false {
            return 0;
        }
    }
}
