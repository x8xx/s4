use std::ffi::c_void;
use std::mem::transmute;
use crate::core::logger::log::*;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::core::network::pktbuf::PktBuf;


#[repr(C)]
pub struct TxArgs {
    pub id: usize,
    pub interface: Interface,
    pub ring: Ring,
    pub batch_count: usize,
    pub pkt_tx_batch_count: usize,
}


pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };
    log!("Init Tx{} Core", tx_args.id);

    // let pipeline_result_list = Array::<&mut PipelineResult>::new(tx_args.batch_count);
    let mut pktbuf_list = Array::<&mut PktBuf>::new(tx_args.batch_count * 2);

    log!("Start Tx{} Core", tx_args.id);
    let mut pkt_count = 0;
    let mut try_count = 0;
    loop {
        let pktbuf_dequeue_count = tx_args.ring.dequeue_burst::<PktBuf>(&pktbuf_list, tx_args.batch_count);
        pkt_count += pktbuf_dequeue_count;
        if pkt_count >= tx_args.pkt_tx_batch_count || try_count >= 3 {
            tx_args.interface.tx(&mut pktbuf_list[0], pkt_count);
            try_count = 0;
            continue;
        }

        try_count += 1;

        if false {
            return 0;
        }
    }
    // 0
}
