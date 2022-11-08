use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::worker::pipeline::PipelineResult;

#[repr(C)]
pub struct TxArgs<'a> {
    pub name: String,
    pub ring: &'a Ring,
    pub batch_count: usize,
}

pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    println!("Start Tx Core");
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };

    let mut pipeline_result_list = Array::<&mut PipelineResult>::new(tx_args.batch_count);
    loop {
        let pipeline_result_dequeue_count = tx_args.ring.dequeue_burst::<PipelineResult>(&mut pipeline_result_list[0], tx_args.batch_count);

        for i in 0..pipeline_result_dequeue_count {
            let pipeline_result = &mut pipeline_result_list[i];
        }
    }
    0
}
