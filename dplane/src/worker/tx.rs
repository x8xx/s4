use std::ffi::c_void;
use std::mem::transmute;
use crate::core::logger::log::*;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::core::network::pktbuf::PktBuf;
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

    // let pipeline_result_list = Array::<&mut PipelineResult>::new(tx_args.batch_count);
    let mut raw_pktbuf_list = Array::<&mut RawPktBuf>::new(tx_args.batch_count);
    // let mut pkt_count: u64 = 0;
    // let mut sum_count: u64 = 0;

    // let mut d_count = 0; 
    // let mut try_count = 0;
    // let mut dequeue_size = tx_args.batch_count;
    // let mut next_dequeue_pos = 0;
    let mut send_count = 0;
    let mut loss = 0;
    loop {
        // let pktbuf_dequeue_count = tx_args.ring.dequeue_burst_resume::<PktBuf>(&pktbuf_list, next_dequeue_pos, dequeue_size);
        let pktbuf_dequeue_count = tx_args.ring.dequeue_burst::<RawPktBuf>(&raw_pktbuf_list, tx_args.batch_count);
        if pktbuf_dequeue_count > 0 {
            let success_count = tx_args.interface.tx(&raw_pktbuf_list, pktbuf_dequeue_count);
            send_count += success_count as u64;
            loss += pktbuf_dequeue_count as u64 - success_count as u64;
            println!("success count {}", success_count);
            println!("send/loss count {}/{}", send_count, loss);
        }


        // next_dequeue_pos += pktbuf_dequeue_count;
        // println!("!!! {}", next_dequeue_pos);

        // if next_dequeue_pos > 0 && (next_dequeue_pos == tx_args.batch_count || try_count == 5) {
        //     println!("??? {}", next_dequeue_pos);
        //     let success_count = tx_args.interface.tx(&mut pktbuf_list[0], next_dequeue_pos);

        //     next_dequeue_pos = 0;
        //     dequeue_size = tx_args.batch_count;
        //     try_count = 0;
        // }

        // try_count += 1;
        // dequeue_size = tx_args.batch_count - next_dequeue_pos;



        // let pktbuf_dequeue_count = tx_args.ring.dequeue_burst::<PktBuf>(&pktbuf_list, tx_args.batch_count);
        // for i in 0..pktbuf_dequeue_count {
        //     let success_count = tx_args.interface.tx(&mut pktbuf_list[i], 1);

            // pkt_count += 1;
            // if pkt_count > 10000 {
            //     sum_count += pkt_count;
            //     pkt_count = 0;
            //     println!("pkt count {}", sum_count);
            // }
            // pktbuf_list[i].free();

        // }

        if false {
            return 0;
        }
    }
    // 0
}
