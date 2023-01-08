use std::ffi::c_void;
use std::mem::transmute;
use crate::core::logger::log::*;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::core::network::pktbuf::RawPktBuf;


#[repr(C)]
pub struct TxArgs {
    pub id: usize,
    pub interface: Interface,
    pub ring: Ring,
    pub batch_count: usize,
}


pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };
    log!("Init Tx{} Core", tx_args.id);

    let mut raw_pktbuf_list = Array::<&mut RawPktBuf>::new(tx_args.batch_count);

    let mut send_count = 0;
    let mut loss_count = 0;
    loop {
        // let pktbuf_dequeue_count = tx_args.ring.dequeue_burst_resume::<PktBuf>(&pktbuf_list, next_dequeue_pos, dequeue_size);
        let pktbuf_dequeue_count = tx_args.ring.dequeue_burst::<RawPktBuf>(&raw_pktbuf_list, tx_args.batch_count);
        if pktbuf_dequeue_count > 0 {
            let success_count = tx_args.interface.tx(&raw_pktbuf_list, pktbuf_dequeue_count);

            send_count += success_count as u64;
            loss_count += pktbuf_dequeue_count as u64 - success_count as u64;
            // println!("success count {}", success_count);
            // println!("send/loss count {}/{}", send_count, loss_count);
        }

        if false {
            return 0;
        }
    }
}
