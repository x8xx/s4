use std::ffi::c_void;
use std::mem::transmute;
use crate::core::runtime::wasm::runtime::new_runtime_args;
use crate::core::runtime::wasm::runtime::RuntimeArgs;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
// use crate::core::memory::ring::RingBuf;
use crate::core::memory::ring::RingBuf;
// use crate::core::network::pktbuf::PktBuf;
use crate::parser::parse_result;
// use crate::parser::parse_result::ParseResult;
use crate::pipeline::pipeline::Pipeline;
use crate::worker::rx::RxResult;

#[repr(C)]
pub struct PipelineArgs {
    pub pipeline: Pipeline,
    pub ring: Ring,
    pub batch_count: usize,
    pub tx_ring_list: Array<Ring>,
    pub cache_crater_ring: Ring,
}


// pub struct PipelineResult {
//     pub rx_result: &'a Rx

// }


pub struct NewCacheElement {


}


pub extern "C" fn start_pipeline(pipeline_args_ptr: *mut c_void) -> i32 {
    println!("Start Pipeline Core");
    let pipeline_args = unsafe { &mut *transmute::<*mut c_void, *mut PipelineArgs>(pipeline_args_ptr) };

    let cache_data_ringbuf = RingBuf::<RuntimeArgs>::new(1024);
    {
        let mut cache_data_array = Array::<&mut RuntimeArgs>:: new(1024); 
        cache_data_ringbuf.malloc_bulk(cache_data_array.as_slice(), cache_data_array.len());
        for i in 0..cache_data_array.len() {
            *cache_data_array[i] = new_runtime_args!(5);
        }
        cache_data_ringbuf.free_bulk(cache_data_array.as_slice(), cache_data_ringbuf.len());
        cache_data_array.free();
    }

    let mut rx_result_list = Array::<&mut RxResult>::new(pipeline_args.batch_count);
    loop {
        let rx_result_dequeue_count = pipeline_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], pipeline_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            let rx_result = &mut rx_result_list[i];
            let cache_data = cache_data_ringbuf.malloc();
            
            pipeline_args.pipeline.run_pipeline((*rx_result).raw_pkt, &mut (*rx_result).parse_result, cache_data);

            if (*rx_result).parse_result.metadata.is_drop {
                (*rx_result).free();
                continue;
            }

            pipeline_args.tx_ring_list[(*rx_result).parse_result.metadata.port as usize].enqueue(*rx_result);

            // cache_creater_ring
        }

        // cache ring
        let cache_result_dequeue_count = pipeline_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], pipeline_args.batch_count);

    }
    0
}
