use std::ffi::c_void;
use std::mem::transmute;
use std::time::Duration;
use std::time::Instant;
use crate::dpdk::interface::Interface;
use crate::dpdk::common::cleanup;
use crate::dpdk::memory::Array;
use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::thread::spawn;
use crate::rx;


#[repr(C)]
pub struct TxArgs {
    pub tap_name: String,
    pub interface: Interface,
    pub execution_time: u64,
    pub rx_args: *mut c_void,
}


pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    println!("start tx thread");
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };

    // let tap_interface = Interface::new(&tx_args.tap_name);
    let tap_interface = Interface::new("net_tap0");

    if !spawn(rx::start_rx, tx_args.rx_args) {
        cleanup();
        panic!("faild start thread rx");
    }

    let mut pktbuf_list = Array::<PktBuf>::new(32);
    // let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(tx_args.execution_time + 5);
    loop {
        let pkt_count = tap_interface.rx(&mut pktbuf_list[0], 32);
        let c = tx_args.interface.tx(&mut pktbuf_list[0], pkt_count);
        // println!("{}", c);

        if end_time < Instant::now() {
            break;
        }
    }


    0 
}
