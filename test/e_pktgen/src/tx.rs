use std::ffi::c_void;
use std::mem::transmute;
use std::time::Duration;
use std::time::Instant;
use std::thread::sleep;
use crate::dpdk::interface::Interface;
use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::common::cleanup;
use crate::dpdk::memory::Array;
use crate::dpdk::memory::Ring;
use crate::dpdk::thread::spawn;
use crate::rx;


#[repr(C)]
pub struct TxArgs {
    // pub tap_name: String,
    pub ring: Ring,
    pub interface: Interface,
    pub pkt_batch_count: usize,
    pub batch_count: usize,
    pub execution_time: u64,
    pub rx_args: *mut c_void,
}


pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };

    println!("init rx thread...");
    if !spawn(rx::start_rx, tx_args.rx_args) {
        cleanup();
        panic!("faild start thread rx");
    }

    let mut pktbuf_list = Array::<&mut Array<PktBuf>>::new(tx_args.batch_count);

    sleep(Duration::from_secs(3));
    let end_time = Instant::now() + Duration::from_secs(tx_args.execution_time + 2);
    let mut counter: u64 = 0;
    println!("start tx thread");
    loop {
        let dequeue_count = tx_args.ring.dequeue_burst::<Array<PktBuf>>(&pktbuf_list, tx_args.batch_count);
        if dequeue_count != 0 {
            println!("dequeue count {}", dequeue_count);
        }
        for i in 0..dequeue_count {
            // sleep(Duration::from_micros(1));
            // sleep(Duration::from_millis(1));
            let c = tx_args.interface.tx(&mut pktbuf_list[i][0], tx_args.pkt_batch_count as u16);
            // if c != 0 {
            //     println!("check :: {}", c);
            // }
            // println!("{}", c);
            counter += c as u64;
        }


        if end_time < Instant::now() {
            println!("generate pkt count: {}", counter);
            break;
        }
    }


    0 
}
