use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use std::time::Duration;
use std::time::Instant;

use crate::dpdk::common::cleanup;
use crate::dpdk::memory::RingBuf;
use crate::dpdk::memory::Array;
use crate::dpdk::memory::Ring;
use crate::dpdk::interface::Interface;
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
    // pub tx_ring: Ring,
    pub interface: Interface,
    // pub tx_args: *mut c_void,
    pub gen_lib_path: String,
}


pub extern "C" fn start_gen(gen_args_ptr: *mut c_void) -> i32 {
    let gen_args = unsafe { &mut *transmute::<*mut c_void, *mut GenArgs>(gen_args_ptr) };

    let pktbuf_pool = PktbufPool::new(8192);
    // let pktbuf_pool = PktbufPool::new(1024);
    let mut pktbuf_list = Array::<PktBuf>::new(gen_args.batch_count);

    let libpktgen = unsafe { libloading::Library::new(&gen_args.gen_lib_path).unwrap() };
    let fn_pktgen = unsafe { libpktgen.get::<libloading::Symbol<unsafe extern fn(buf_list: *mut u8, state: *mut c_void) -> *mut c_void>>(b"pktgen").unwrap() };
    let mut state = null_mut();


    println!("start gen thread");
    let end_time = Instant::now() + Duration::from_secs(gen_args.execution_time + 3);
    let mut c = 0;
    loop {
        // println!("pktbuf alloc");
        if !pktbuf_pool.alloc_bulk(pktbuf_list.clone()) {
            if end_time < Instant::now() {
                println!("gen end");
                return 0;
            }

            continue;
        }

        // println!("pktbuf custom");
        for i in 0..pktbuf_list.len() {
            pktbuf_list[i].append(1500);
            c += 1;
            // println!("check {}", c);
            let (pkt, _) = pktbuf_list[i].get_raw_pkt();
            state = unsafe { fn_pktgen(pkt, state) };
        }

        gen_args.interface.tx(&mut pktbuf_list[0], gen_args.batch_count as u16);

        c  += gen_args.interface.tx(&mut pktbuf_list[0], gen_args.batch_count as u16);
            // println!("gen pkt {} from {}", c, gen_args.interface.queue_number);
        // std::thread::sleep(Duration::from_millis(10));

        // gen_args.tx_ring.enqueue(&mut pktbuf_list);


        if end_time < Instant::now() {
            println!("gen end");
            return 0;
        }


        if false {
            return 0;
        }
    }
}
