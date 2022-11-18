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

    let pkt_ringbuf = RingBuf::<Array<u8>>::new(4096);
    {
        let ptr_array = Array::<&mut Array<u8>>:: new(pkt_ringbuf.len()); 
        pkt_ringbuf.malloc_bulk(ptr_array.as_slice(), ptr_array.len());
        for (_, element) in ptr_array.as_slice().iter_mut().enumerate() {
            **element = Array::<u8>::new(64);
        }
        pkt_ringbuf.free_bulk(ptr_array.as_slice(), ptr_array.len());
        ptr_array.free();
    }

    if !spawn(tx::start_tx, gen_args.tx_args) {
        cleanup();
        panic!("faild start thread tx");
    }


    // let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(gen_args.execution_time + 10);
    loop {
        let buf = pkt_ringbuf.malloc();
        tap_tx.send_to(buf.as_slice(), None);
        pkt_ringbuf.free(buf);

        if end_time < Instant::now() {
            break;
        }
    }

    0
}
