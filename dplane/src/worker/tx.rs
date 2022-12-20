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
    // let mut pkt_count = 0;
    let mut d_count = 0; 
    let mut try_count = 0;
    let mut next_dequeue_pos = 0;
    loop {
        let pktbuf_dequeue_count = tx_args.ring.dequeue_burst_resume::<PktBuf>(&pktbuf_list, next_dequeue_pos, tx_args.batch_count);
        next_dequeue_pos += pktbuf_dequeue_count;
        // pkt_count += pktbuf_dequeue_count;
        // if next_dequeue_pos >= tx_args.pkt_tx_batch_count || try_count >= 10 {
        if next_dequeue_pos >= tx_args.pkt_tx_batch_count || true {
            // d_count += next_dequeue_pos;
            // if next_dequeue_pos != 0 {
            //     println!("d_count: {}", d_count);

            // }

            // if d_count == 100000 {
            //     panic!("end");
            // }
            let success_count = tx_args.interface.tx(&mut pktbuf_list[0], next_dequeue_pos);
            if success_count != 0 {
                // log!("Tx{} success send packet {}", tx_args.id, success_count);
                debug_log!("n pos: {}", next_dequeue_pos);
                debug_log!("Tx{} success send packet {}", tx_args.id, success_count);
            }
            try_count = 0;
            next_dequeue_pos = 0;
            continue;
        }

        try_count += 1;

        if false {
            return 0;
        }
    }
    // 0
}
