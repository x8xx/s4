use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::core::network::pktbuf::PktBuf;
use crate::worker::rx::RxResult;
// use crate::worker::pipeline::PipelineResult;

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
    println!("Init Tx{} Core", tx_args.id);

    // let pipeline_result_list = Array::<&mut PipelineResult>::new(tx_args.batch_count);
    let mut pktbuf_list = Array::<&mut PktBuf>::new(tx_args.batch_count);

    println!("Start Tx{} Core", tx_args.id);
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

//         for i in 0..pipeline_result_dequeue_count {
//             let PipelineResult { owner_ring: _, rx_result, } = pipeline_result_list.get(i);
//             let rx_result = unsafe { &mut **rx_result };

//             tx_args.interface.tx(&mut rx_result.pktbuf);
//             rx_result.free();
//             rx_result.pktbuf.free();
//             pipeline_result_list.get(i).free();
//         }


        if false {
            return 0;
        }
    }
    // 0
}
