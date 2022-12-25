use std::ffi::c_void;
use std::mem::transmute;
use std::time::Duration;
use std::time::Instant;
use crate::dpdk::pktbuf::PktBuf;
use crate::dpdk::memory::Array;
use crate::dpdk::interface::Interface;


#[repr(C)]
pub struct RxArgs {
    // pub interface: Interface,
    pub port_number: u16,
    pub queue: u16,
    pub execution_time: u64,
    pub result: *mut u64,
}


pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    println!("start rx thread");
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    // let interface = Interface {
    //     port_number: rx_args.port_number,
    //     queue_number: 0,
    // };

    let mut pktbuf_list = Array::<PktBuf>::new(32);

    let start_time = Instant::now();
    let end_time = Instant::now() + Duration::from_secs(rx_args.execution_time);
    
    let mut counter: u64 = 0;

    // let mut interfaces = Array::new(rx_args.max_rx_queues as usize);
    // for i in 0..interfaces.len() {
    //     interfaces.init(i, Interface {
    //         port_number: rx_args.port_number,
    //         queue_number: i as u16,
    //     });
    // }
    let interface = Interface {
        port_number: rx_args.port_number,
        queue_number: rx_args.queue,
    };
    loop {
        // for i in 0..interfaces.len() {
            let pkt_count = interface.rx(&mut pktbuf_list[0], 32);
            // println!("? {}", pkt_count);
            counter += pkt_count as u64;
            pktbuf_list[0].free(pkt_count as u32);

            // let now = Instant::now();
            if end_time < Instant::now() {
                println!("execution time: {}", (Instant::now() - start_time).as_secs());
                println!("{} receive pkt count: {}", rx_args.queue, counter);
                // panic!("success");
                unsafe {
                    *rx_args.result = counter;
                }
                return 0;
            }
        // }

        if false {
            return 0;
        }
    }
}
