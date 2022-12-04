use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use std::time::Duration;
use std::time::Instant;
use pnet::datalink;
use pnet::datalink::NetworkInterface;
use pnet::datalink::DataLinkSender;
use pnet::datalink::Channel::Ethernet;

use crate::dpdk::common::cleanup;
use crate::dpdk::memory::RingBuf;
use crate::dpdk::memory::Array;
use crate::dpdk::memory::Ring;
use crate::dpdk::pktbuf::PktbufPool;
use crate::dpdk::pktbuf::PktBuf;
// use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::thread::spawn;
use crate::tx;


#[repr(C)]
pub struct GenArgs {
    pub tap_name: String,
    pub batch_count: usize,
    pub execution_time: u64,
    pub tx_ring: Ring,
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
    let gen_args = unsafe { &mut *transmute::<*mut c_void, *mut GenArgs>(gen_args_ptr) };

    // launch tx thread
    if !spawn(tx::start_tx, gen_args.tx_args) {
        cleanup();
        panic!("faild start thread tx");
    }


    println!("-");
    let pktbuf_pool = PktbufPool::new(8192);
    println!("-=-===");
    let mut pktbuf_list = Array::<PktBuf>::new(gen_args.batch_count);

    // let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(gen_args.execution_time + 3);

    let libpktgen = unsafe { libloading::Library::new(&gen_args.gen_lib_path).unwrap() };
    let fn_pktgen = unsafe { libpktgen.get::<libloading::Symbol<unsafe extern fn(buf_list: *mut u8, state: *mut c_void) -> *mut c_void>>(b"pktgen").unwrap() };
    let mut state = null_mut();

    // launch tx thread
    if !spawn(tx::start_tx, gen_args.tx_args) {
        cleanup();
        panic!("faild start thread tx");
    }

    println!("start gen thread");
    loop {
        if !pktbuf_pool.alloc_bulk(pktbuf_list.clone()) {
            if end_time < Instant::now() {
                return 0;
            }

            continue;
        }

        for i in 0..pktbuf_list.len() {
            let (pkt, _) = pktbuf_list[i].get_raw_pkt();
            state = unsafe { fn_pktgen(pkt, state) };
        }

        gen_args.tx_ring.enqueue(&mut pktbuf_list[0]);


        if end_time < Instant::now() {
            return 0;
        }


        if false {
            return 0;
        }
    }
}
