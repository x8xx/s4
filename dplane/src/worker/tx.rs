use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::worker::rx::RxResult;
use crate::worker::pipeline::PipelineResult;

#[repr(C)]
pub struct TxArgs {
    pub interface: Interface,
    pub ring: Ring,
    pub batch_count: usize,
}

pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    println!("Start Tx Core");
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };

    let pipeline_result_list = Array::<&mut PipelineResult>::new(tx_args.batch_count);
    loop {
        let pipeline_result_dequeue_count = tx_args.ring.dequeue_burst::<PipelineResult>(&pipeline_result_list, tx_args.batch_count);
        for i in 0..pipeline_result_dequeue_count {
            let PipelineResult { owner_ring: _, rx_result, tx_conf } = pipeline_result_list.get(i);
            let rx_result = unsafe { &mut **rx_result };

            tx_args.interface.tx(&mut rx_result.pktbuf);
            rx_result.free();
            pipeline_result_list.get(i).free();
        }


        if false {
            return 0;
        }
    }
    // 0
}
