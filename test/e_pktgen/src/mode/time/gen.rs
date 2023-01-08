use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;

use crate::dpdk::memory::Array;
use crate::dpdk::memory::Locker;
use crate::dpdk::interface::Interface;
use crate::dpdk::pktbuf::PktbufPool;
use crate::dpdk::pktbuf::PktBuf;


#[repr(C)]
pub struct GenArgs {
    pub batch_count: usize,
    pub interface: Interface,
    pub gen_lib_path: String,
    pub start_locker: Option<Locker>,
    pub end_locker: Locker,
}


pub extern "C" fn start_gen(gen_args_ptr: *mut c_void) -> i32 {
    let gen_args = unsafe { &mut *transmute::<*mut c_void, *mut GenArgs>(gen_args_ptr) };

    let pktbuf_pool = PktbufPool::new(8192);
    let mut pktbuf_list = Array::<PktBuf>::new(gen_args.batch_count);


    let libpktgen = unsafe { libloading::Library::new(&gen_args.gen_lib_path).unwrap() };
    let fn_pktgen = unsafe { libpktgen.get::<libloading::Symbol<unsafe extern fn(buf_list: *mut u8, state: *mut c_void) -> *mut c_void>>(b"pktgen").unwrap() };
    let mut state = null_mut();

    if !gen_args.start_locker.is_none() {
        gen_args.start_locker.unwrap().unlock();
    }


    let mut counter = 0;
    // let mut loss_counter = 0;
    // let mut skip_counter = 0;
    println!("start gen thread");
    loop {
        if !pktbuf_pool.alloc_bulk(pktbuf_list.clone()) {
            // skip_counter += 1;
            if gen_args.end_locker.check() {
                println!("{} generate pkt count: {}", gen_args.interface.queue_number, counter);
                // println!("{} failed pkt count: {}", gen_args.interface.queue_number, skip_counter);
                // println!("{} loss pkt count: {}", gen_args.interface.queue_number, loss_counter);
                pktbuf_pool.free();
                pktbuf_list.free();
                return 0;
            }
            continue;
        }

        for i in 0..pktbuf_list.len() {
            pktbuf_list[i].append(1500);
            let (pkt, _) = pktbuf_list[i].get_raw_pkt();
            state = unsafe { fn_pktgen(pkt, state) };
        }

        let success_count = gen_args.interface.tx(&mut pktbuf_list[0], gen_args.batch_count as u16) as u64;
        let loss_count = gen_args.batch_count as u64 - success_count;
        counter += success_count;
        // loss_counter += loss_count;
        if loss_count > 0 {
            pktbuf_list[success_count as usize].free(loss_count as u32);
        }
        

        if gen_args.end_locker.check() {
            println!("{} generate pkt count: {}", gen_args.interface.queue_number, counter);
            // println!("{} loss pkt count: {}", gen_args.interface.queue_number, loss_counter);
            // println!("{} failed pkt count: {}", gen_args.interface.queue_number, skip_counter);
            pktbuf_pool.free();
            pktbuf_list.free();
            return 0;
        }


        if false {
            return 0;
        }
    }
}
