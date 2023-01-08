use std::ffi::c_void;
use std::mem::transmute;
use std::time::Duration;
use std::time::Instant;
use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::memory::Array;
use crate::dpdk::memory::Locker;
use crate::dpdk::interface::Interface;


#[repr(C)]
pub struct RxArgs {
    // pub interface: Interface,
    pub port_number: u16,
    pub queue: u16,
    pub execution_time: u64,
    pub start_locker: Locker,
    pub end_locker: Locker,
}


pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    let mut pktbuf_list = Array::<PktBuf>::new(8192);

    let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(rx_args.execution_time);
    
    let mut counter: u64 = 0;

    let interface = Interface {
        port_number: rx_args.port_number,
        queue_number: rx_args.queue,
    };

    rx_args.start_locker.wait();

    println!("start rx thread");
    loop {
        // for i in 0..interfaces.len() {
            let pkt_count = interface.rx(&mut pktbuf_list[0], 1024);
            // println!("? {}", pkt_count);
            counter += pkt_count as u64;
            pktbuf_list[0].free(pkt_count as u32);

            // let now = Instant::now();
            if end_time < Instant::now() {
                rx_args.end_locker.unlock();
                pktbuf_list.free();
                println!("execution time: {}", (Instant::now() - start_time).as_secs());
                println!("{} receive pkt count: {}", rx_args.queue, counter);
                return 0;
            }
        // }

        if false {
            return 0;
        }
    }
}
