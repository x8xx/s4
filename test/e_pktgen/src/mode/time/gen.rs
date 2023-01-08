use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
// use std::time::Duration;
// use std::time::Instant;

use crate::dpdk::memory::Array;
use crate::dpdk::memory::Locker;
use crate::dpdk::interface::Interface;
use crate::dpdk::pktbuf::PktbufPool;
use crate::dpdk::pktbuf::PktBuf;


#[repr(C)]
pub struct GenArgs {
    pub batch_count: usize,
    // pub execution_time: u64,
    pub interface: Interface,
    pub gen_lib_path: String,
    pub start_locker: Option<Locker>,
    pub end_locker: Locker,
}


pub extern "C" fn start_gen(gen_args_ptr: *mut c_void) -> i32 {
    let gen_args = unsafe { &mut *transmute::<*mut c_void, *mut GenArgs>(gen_args_ptr) };

    let pktbuf_pool = PktbufPool::new(8192);
    // let pktbuf_pool = PktbufPool::new(1024);
    let mut pktbuf_list = Array::<PktBuf>::new(gen_args.batch_count);


    let libpktgen = unsafe { libloading::Library::new(&gen_args.gen_lib_path).unwrap() };
    let fn_pktgen = unsafe { libpktgen.get::<libloading::Symbol<unsafe extern fn(buf_list: *mut u8, state: *mut c_void) -> *mut c_void>>(b"pktgen").unwrap() };
    let mut state = null_mut();

    if !gen_args.start_locker.is_none() {
        gen_args.start_locker.unwrap().unlock();
    }

    println!("start gen thread");
    // let end_time = Instant::now() + Duration::from_secs(gen_args.execution_time + 3);
    let mut counter = 0;
    loop {
        // println!("pktbuf alloc");
        if !pktbuf_pool.alloc_bulk(pktbuf_list.clone()) {
            if gen_args.end_locker.check() {
                println!("{} generate pkt count: {}", gen_args.interface.queue_number, counter);
                pktbuf_pool.free();
                pktbuf_list.free();
                return 0;
            }
            continue;
        }

        // println!("pktbuf custom");
        for i in 0..pktbuf_list.len() {
            pktbuf_list[i].append(1500);
            let (pkt, _) = pktbuf_list[i].get_raw_pkt();
            state = unsafe { fn_pktgen(pkt, state) };
        }

        counter += gen_args.interface.tx(&mut pktbuf_list[0], gen_args.batch_count as u16) as u64;


        let mut result = false;
        let mut resul_ptrt: &mut bool = &mut result;
        // if end_time < Instant::now() {
        if gen_args.end_locker.check() {
            println!("{} generate pkt count: {}", gen_args.interface.queue_number, counter);
            pktbuf_pool.free();
            pktbuf_list.free();
            return 0;
        }


        if false {
            return 0;
        }
    }
}
