use std::ffi::c_void;
use std::mem::transmute;
// use crate::parser::parse_result::ParseResult;
use crate::pipeline::pipeline::Pipeline;
use crate::worker::rx::RxResult;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
// use crate::core::memory::ring::RingBuf;
// use crate::core::network::pktbuf::PktBuf;

#[repr(C)]
pub struct PipelineArgs<'a> {
    pub pipeline: Pipeline<'a>,
    pub ring: &'a Ring,
    pub batch_count: usize,
}


pub extern "C" fn start_pipeline(pipeline_args_ptr: *mut c_void) -> i32 {
    println!("Start Pipeline Core");
    let pipeline_args = unsafe { &mut *transmute::<*mut c_void, *mut PipelineArgs>(pipeline_args_ptr) };

    // let mut rx_result_list = Array::<*mut RxResult>::new(pipeline_args.batch_count);
    let mut rx_result_list = Array::<&mut RxResult>::new(pipeline_args.batch_count);
    loop {
        let rx_result_dequeue_count = pipeline_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], pipeline_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            let rx_result = &rx_result_list[i];
            println!("hdr len {}", (*rx_result).parse_result.hdr_len);
        }
    }
    0
}
