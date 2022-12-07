use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::heap::Heap;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::ring::init_ringbuf_element;
use crate::core::network::pktbuf::PktBuf;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::output::Output;
use crate::worker::rx::RxResult;
use crate::worker::rx::HashCalcResult;
use crate::worker::cache::CacheResult;


#[repr(C)]
pub struct PipelineArgs {
    pub id: usize,
    pub pipeline: Pipeline,
    pub ring_from_rx: Ring,
    pub ring_from_cache: Ring,

    pub batch_count: usize,
    pub table_list_len: usize,
    pub header_max_size: usize,
    pub buf_size: usize,

    pub tx_ring_list: Array<Ring>,
    pub cache_creater_ring: Ring,
}


// pub struct PipelineResult {
//     pub owner_ring: *mut RingBuf<PipelineResult>,
//     pub rx_result: *mut RxResult,
// }


// impl PipelineResult {
//     pub fn free(&mut self) {
//         unsafe {
//             (*self.owner_ring).free(self);
//         }
//     }
// }


pub struct NewCacheElement {
    pub owner_ring: *mut RingBuf<NewCacheElement>,
    pub rx_id: usize,
    pub l1_key: Array<u8>,
    pub l1_key_len: usize,
    pub cache_id: usize,
    pub cache_data: CacheData,
    pub parse_result: *mut ParseResult,
    pub hash_calc_result: *mut HashCalcResult
}

impl NewCacheElement {
    pub fn free(&mut self) {
        unsafe {
            (*self.owner_ring).free(self);
        }
    }
}


pub fn output_pkt(tx_ring_list: &Array<Ring>, pktbuf: &mut PktBuf, output: Output) -> bool {
    match output {
        Output::Port(port_num) => {
            tx_ring_list[port_num as usize].enqueue(pktbuf);
        },
        Output::All => {
            // TODO
        },
        Output::Controller => {
            tx_ring_list[0].enqueue(pktbuf);
        },
        Output::Drop => {
            return false;
        },
    }
    true
}


pub extern "C" fn start_pipeline(pipeline_args_ptr: *mut c_void) -> i32 {
    let pipeline_args = unsafe { &mut *transmute::<*mut c_void, *mut PipelineArgs>(pipeline_args_ptr) };
    println!("Init Pipeline{} Core", pipeline_args.id);


    // init ringbuf (pipeline result)
    // let mut pipeline_result_ringbuf = RingBuf::<PipelineResult>::new(pipeline_args.buf_size);
    // init_ringbuf_element!(pipeline_result_ringbuf, PipelineResult, {
    //     owner_ring => &mut pipeline_result_ringbuf as *mut RingBuf<PipelineResult>,
    // });

    let mut heap = Heap::new(pipeline_args.buf_size * (pipeline_args.header_max_size + pipeline_args.table_list_len));

    // init ringbuf (new cache)
    let mut new_cache_element_ringbuf = RingBuf::<NewCacheElement>::new(pipeline_args.buf_size);
    init_ringbuf_element!(new_cache_element_ringbuf, NewCacheElement, {
        owner_ring => &mut new_cache_element_ringbuf as *mut RingBuf<NewCacheElement>,
        // l1_key => Array::new(pipeline_args.header_max_size),
        // cache_data => Array::new(pipeline_args.table_list_len),
        l1_key => heap.malloc(pipeline_args.header_max_size),
        cache_data => heap.malloc(pipeline_args.table_list_len),
    });


    let rx_result_list = Array::<&mut RxResult>::new(pipeline_args.batch_count);
    let cache_result_list = Array::<&mut CacheResult>::new(pipeline_args.batch_count);

    println!("Start Pipeline{} Core", pipeline_args.id);
    loop {
        // from rx (through cache core)
        let rx_result_dequeue_count = pipeline_args.ring_from_rx.dequeue_burst::<RxResult>(&rx_result_list, pipeline_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            println!("pp: rx result");
            let rx_result = rx_result_list.get(i);
            // println!("pipeline malloc");
            // let mut pipeline_result = pipeline_result_ringbuf.malloc();
            // println!("pipeline malloc ok");
            // pipeline_result.rx_result = *rx_result as *mut RxResult;

            let RxResult {
                owner_ring: _,
                id: _,
                pktbuf,
                raw_pkt: _,
                pkt_len,
                parse_result,
                cache_data,
                hash_calc_result: _,
            } = rx_result_list.get(i);

            let mut output = Output::Drop;
            pipeline_args.pipeline.run_cache_pipeline(rx_result_list.get(i).raw_pkt, *pkt_len,  parse_result, cache_data, &mut output);

           if !output_pkt(&pipeline_args.tx_ring_list, pktbuf, output) {
               println!("pp drop");
                rx_result_list.get(i).free();
                pktbuf.free();
                continue;
           } 

           rx_result_list.get(i).free();
        }


        // // from cache
        let cache_result_dequeue_count = pipeline_args.ring_from_cache.dequeue_burst::<CacheResult>(&cache_result_list, pipeline_args.batch_count);
        for i in 0..cache_result_dequeue_count {
            println!("pp: cache result");

            let CacheResult {
                owner_ring: _,
                rx_result,
                id: ref cache_id,
                is_cache_hit: ref is_cache_hit,
                cache_data: ref cache_data, 
            } = cache_result_list.get(i);

            // pipeline_result.rx_result = *rx_result;

            let RxResult {
                owner_ring: _,
                id: ref rx_id,
                pktbuf,
                raw_pkt,
                pkt_len,
                parse_result: ref parse_result,
                cache_data: ref mut cache_data,
                hash_calc_result,
            } = unsafe { &mut **rx_result };

            let mut output = Output::Drop;


            // cache
            if *is_cache_hit {
                pipeline_args.pipeline.run_cache_pipeline(*raw_pkt, *pkt_len, parse_result, cache_data, &mut output);
                cache_result_list.get(i).free();
                unsafe { (**hash_calc_result).free() };
            // no cache
            } else {
                println!("new_cache malloc");
                let new_cache_element = new_cache_element_ringbuf.malloc();
                println!("new_cache malloc ok");
                pipeline_args.pipeline.run_pipeline(*raw_pkt, *pkt_len, parse_result, &mut new_cache_element.cache_data, &mut output);

                new_cache_element.l1_key_len = parse_result.hdr_size;
                for i in 0..new_cache_element.l1_key_len {
                    unsafe {
                        new_cache_element.l1_key[i] = *raw_pkt.offset(i as isize);
                    }
                }
                new_cache_element.rx_id = *rx_id;
                new_cache_element.cache_id = *cache_id;
                new_cache_element.hash_calc_result = *hash_calc_result as *const HashCalcResult as *mut HashCalcResult;

                // to cache_creater (main core)
                pipeline_args.cache_creater_ring.enqueue(new_cache_element);
            }


           if !output_pkt(&pipeline_args.tx_ring_list, pktbuf, output) {
               println!("pp: drop");
               rx_result_list.get(i).free();
               pktbuf.free();
               continue;
           } 

           rx_result_list.get(i).free();
        }

        if false {
            return 0;
        }
    }
    // 0
}
