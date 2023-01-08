use std::ffi::c_void;
use std::mem::transmute;
use std::sync::RwLock;
use crate::core::logger::log::log;
use crate::core::logger::log::debug_log;
use crate::core::memory::ring::Ring;
use crate::core::memory::array::Array;
use crate::cache::cache::CacheElement;
use crate::cache::cache::CacheData;
use crate::cache::tss::TupleSpace;
use crate::cache::tss::KeyStore;
use crate::worker::rx::Pkt;
// use crate::worker::rx::HashCalcResult;


#[repr(C)]
pub struct CacheArgs<'a> {
    pub id: usize,
    pub ring: Ring,

    pub batch_count: usize,
    pub buf_size: usize,
    pub header_max_size: usize,

    // cache
    pub l2_cache: Array<RwLock<CacheElement>>,
    pub l3_cache: &'a TupleSpace<'a>,
    // pub l3_cache: Arc<TupleSpace<'a>>,

    // ring list
    pub pipeline_ring_list: Array<Ring>,
}


fn next_core(mut current_core: usize, core_limit: usize) -> usize {
    current_core += 1;
    if current_core == core_limit {
        current_core = 0;
    }
    current_core
}


pub extern "C" fn start_cache(cache_args_ptr: *mut c_void) -> i32 {
    let cache_args = unsafe { &mut *transmute::<*mut c_void, *mut CacheArgs>(cache_args_ptr) };
    log!("Init Cache{} Core", cache_args.id);

    let pkt_list = Array::<&mut Pkt>::new(cache_args.batch_count);

    let mut next_pipeline_core = 0;
    let mut tss_key_store = KeyStore {
        key: Array::new(cache_args.header_max_size),
        key_len: 0,
    };

    log!("Start Cache{} Core", cache_args.id);
    loop {
        let pkt_dequeue_count = cache_args.ring.dequeue_burst::<Pkt>(&pkt_list, cache_args.batch_count);
        for i in 0..pkt_dequeue_count {

            let pkt = pkt_list.get(i);
            let pkt_analysis_result = &mut pkt.pkt_analysis_result;

            pkt_analysis_result.cache_id = cache_args.id;
            pkt_analysis_result.is_cache_hit = false;


            if pkt_analysis_result.is_lbf_hit {
                debug_log!("Cache{} Check L2 Cache", cache_args.id);
                // l2 cache
                let cache_element = cache_args.l2_cache[pkt_analysis_result.l2_hash as usize].read().unwrap();
                if cache_element.cmp_ptr_key(pkt_analysis_result.l2_key.as_ptr(), pkt_analysis_result.l2_key_len as isize) {
                    debug_log!("Cache{} Hit L2 Cache", cache_args.id);
                    pkt_analysis_result.cache_data = cache_element.data.clone();
                    pkt_analysis_result.is_cache_hit = true;

                    debug_log!("Cache{} enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);
                    if cache_args.pipeline_ring_list[next_pipeline_core].enqueue(*pkt_list.get(i)) < 0 {
                        debug_log!("Cache{} failed enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);
                        // pkt_analysis_result.pktbuf.free();
                        // unsafe { (*pkt_analysis_result.hash_calc_result).free(); };
                        // hash_calc_result.free();
                        (*pkt_analysis_result).free();
                        (*pkt).free();
                        // cache_result.free();
                        continue;
                    }
                    debug_log!("Cache{} complete enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);
                    next_pipeline_core = next_core(next_pipeline_core, cache_args.pipeline_ring_list.len());
                    continue;
                }
                debug_log!("Cache{} No Hit L2 Cache", cache_args.id);
            }


            // l3 cache (tss)
            // let cache_data = cache_args.l3_cache.search(pkt_analysis_result.raw_pkt, &mut tss_key_store);
            // match cache_data {
            //     Some(cache_data) => {
            //         cache_result.cache_data = cache_data;
            //         cache_result.is_cache_hit = true;
            //     },
            //     None => {},
            // }


            debug_log!("Cache{} enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);
            if cache_args.pipeline_ring_list[next_pipeline_core].enqueue(*pkt_list.get(i)) < 0 {
                debug_log!("Cache{} failed enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);
                // pkt_analysis_result.pktbuf.free();
                // hash_calc_result.free();
                (*pkt_analysis_result).free();
                (*pkt).free();
                continue;
            }
            debug_log!("Cache{} complete enqueue to Pipeline Core {}", cache_args.id, next_pipeline_core);


            next_pipeline_core = next_core(next_pipeline_core, cache_args.pipeline_ring_list.len());
            continue;
        }


        if false {
            return 0;
        }
    }
}
