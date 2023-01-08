use std::ffi::c_void;
use std::mem::transmute;
use crate::core::logger::log::log;
use crate::core::logger::log::debug_log;
// use crate::core::memory::heap::Heap;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
// use crate::core::memory::ring::RingBuf;
// use crate::core::memory::ring::init_ringbuf_element;
use crate::core::network::pktbuf::PktBuf;
// use crate::parser::parse_result::ParseResult;
// use crate::cache::cache::CacheData;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::output::Output;
use crate::worker::rx::Pkt;
use crate::worker::rx::PktAnalysisResult;
// use crate::worker::rx::HashCalcResult;
// use crate::worker::cache::CacheResult;


#[repr(C)]
pub struct PipelineArgs {
    pub id: usize,
    pub pipeline: Pipeline,
    pub ring_from_rx: Ring,

    pub batch_count: usize,
    // pub table_list_len: usize,
    // pub header_max_size: usize,
    // pub buf_size: usize,

    pub tx_ring_list: Array<Array<Ring>>,
    pub cache_creater_ring: Ring,
}


// pub struct NewCacheElement<'a> {
//     pub owner_ring: *mut RingBuf<NewCacheElement<'a>>,

//     pub rx_id: usize,
//     pub cache_id: usize,

//     pub l1_key: Array<u8>,
//     pub l1_key_len: usize,

//     pub cache_data: CacheData,

//     pub parse_result: *mut ParseResult,
//     pub hash_calc_result: *mut HashCalcResult
// }

// impl<'a> NewCacheElement<'a> {
//     pub fn free(&mut self) {
//         unsafe {
//             (*self.owner_ring).free(self);
//         }
//     }
// }


fn output_pkt(tx_ring_list: &Array<Array<Ring>>, next_tx_queue_list: &mut Array<usize>, pktbuf: &mut PktBuf, output: Output) -> bool {
    match output {
        Output::Port(port_num) => {
            debug_log!("Pipeline enqueue to Tx{}", port_num);
            if tx_ring_list[port_num as usize][next_tx_queue_list[port_num as usize]].enqueue(pktbuf.as_raw()) < 0 {
                return false;
            }
            next_tx_queue_list[port_num as usize] += 1;
            if next_tx_queue_list[port_num as usize] >= tx_ring_list[port_num as usize].len() {
                next_tx_queue_list[port_num as usize] = 0;
            }
        },
        Output::All => {
            // TODO
        },
        Output::Controller => {
            debug_log!("Pipeline enqueue to Tx0 (CPU)");
            // if tx_ring_list[0][0].enqueue(pktbuf.as_raw()) < 0 {
            //     return false;
            // }
        },
        Output::Drop => {
            return false;
        },
    }
    true
}


pub extern "C" fn start_pipeline(pipeline_args_ptr: *mut c_void) -> i32 {
    let pipeline_args = unsafe { &mut *transmute::<*mut c_void, *mut PipelineArgs>(pipeline_args_ptr) };
    log!("Init Pipeline{} Core", pipeline_args.id);

    // let mut heap = Heap::new(pipeline_args.buf_size * (pipeline_args.header_max_size + pipeline_args.table_list_len));

    // // init ringbuf (new cache)
    // let mut new_cache_element_ringbuf = RingBuf::<NewCacheElement>::new(pipeline_args.buf_size);
    // init_ringbuf_element!(new_cache_element_ringbuf, NewCacheElement, {
    //     owner_ring => &mut new_cache_element_ringbuf as *mut RingBuf<NewCacheElement>,
    //     l1_key => heap.malloc(pipeline_args.header_max_size),
    //     cache_data => heap.malloc(pipeline_args.table_list_len),
    // });



    let pkt_list = Array::<&mut Pkt>::new(pipeline_args.batch_count);
    // let cache_result_list = Array::<&mut CacheResult>::new(pipeline_args.batch_count);

    let mut next_tx_queue_list = Array::<usize>::new(pipeline_args.tx_ring_list.len());

    log!("Start Pipeline{} Core", pipeline_args.id);
    loop {
        let pkt_dequeue_count = pipeline_args.ring_from_rx.dequeue_burst::<Pkt>(&pkt_list, pipeline_args.batch_count);
        for i in 0..pkt_dequeue_count {
            debug_log!("Pipeline{} dequeue pkt_analysis_result", pipeline_args.id);

            // let pkt_analysis_result = pkt_analysis_result_list.get(i);

            let pkt = pkt_list.get(i);
            let PktAnalysisResult {
                owner_ring: _,
                rx_id: _,
                cache_id: _,
                // pktbuf,
                // raw_pkt,
                // pkt_len,
                parse_result,
                cache_data,
                is_cache_hit,
                // hash_calc_result,
                l1_key,
                l1_key_len,
                l1_hash: _,
                l2_key: _,
                l2_key_len: _,
                l2_hash: _,
                is_lbf_hit: _,
            } = &mut pkt.pkt_analysis_result;


            let mut output = Output::Drop;
            if *is_cache_hit {
                debug_log!("Pipeline{} run cache pipeline", pipeline_args.id);
                pipeline_args.pipeline.run_cache_pipeline(pkt.raw_pkt, pkt.len,  parse_result, cache_data, &mut output);
                debug_log!("Pipeline{} complete cache pipeline", pipeline_args.id);
                pkt.pkt_analysis_result.free();
            } else {
                // debug_log!("Pipeline{} new_cache_element malloc", pipeline_args.id);
                // let new_cache_element = new_cache_element_ringbuf.malloc();
                // debug_log!("Pipeline{} done new_cache_elementmalloc", pipeline_args.id);

                debug_log!("Pipeline{} run pipeline", pipeline_args.id);
                pipeline_args.pipeline.run_pipeline(pkt.raw_pkt, pkt.len, parse_result, cache_data, &mut output);
                debug_log!("Pipeline{} complete pipeline", pipeline_args.id);

                *l1_key_len = parse_result.hdr_size;
                for i in 0..*l1_key_len {
                    unsafe {
                        l1_key[i] = *pkt.raw_pkt.offset(i as isize);
                    }
                }
                // rx_id = *rx_id;
                // new_cache_element.cache_id = *cache_id;
                // new_cache_element.hash_calc_result = *hash_calc_result as *const HashCalcResult as *mut HashCalcResult;

                debug_log!("Pipeline{} enqueue to cache_creater_ring", pipeline_args.id);
                // to cache_creater (main core)
                pipeline_args.cache_creater_ring.enqueue(&mut *pkt.pkt_analysis_result);
            }


            if !output_pkt(&pipeline_args.tx_ring_list, &mut next_tx_queue_list, &mut pkt.pktbuf, output) {
                debug_log!("Pipeline{} drop pkt", pipeline_args.id);
                pkt.pktbuf.free();
                continue;
            } 

            pkt.free();
            // pkt_analysis_result_list.get(i).free();
        }


        if false {
            return 0;
        }
    }
}
