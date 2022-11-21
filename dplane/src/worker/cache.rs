use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::ring::init_ringbuf_element;
use crate::core::memory::array::Array;
use crate::cache::cache::CacheElement;
use crate::cache::cache::CacheData;
use crate::cache::tss::TupleSpace;
use crate::cache::tss::KeyStore;
use crate::worker::rx::RxResult;
// use crate::worker::rx::HashCalcResult;


#[repr(C)]
pub struct CacheArgs<'a> {
    pub id: usize,
    pub ring: Ring,

    pub batch_count: usize,
    pub buf_len: usize,
    pub hdr_max_len: usize,

    // cache
    pub l2_cache: Array<CacheElement>,
    pub l3_cache: &'a TupleSpace<'a>,

    // ring list
    pub pipeline_ring_list: Array<Ring>,
}


pub struct CacheResult {
    pub owner_ring: *mut RingBuf<CacheResult>,
    pub id: usize,
    pub rx_result: *mut RxResult,
    pub cache_data: CacheData,
    pub is_cache_hit: bool,
}

impl CacheResult {
    pub fn free(&mut self) {
        unsafe {
            (*self.owner_ring).free(self);
        }
    }
}


fn next_core(mut current_core: usize, core_limit: usize) -> usize {
    current_core += 1;
    if current_core == core_limit {
        current_core = 0;
    }
    current_core
}


pub extern "C" fn start_cache(cache_args_ptr: *mut c_void) -> i32 {
    println!("Start Cache Core");
    let cache_args = unsafe { &mut *transmute::<*mut c_void, *mut CacheArgs>(cache_args_ptr) };

    // init ringbuf
    let mut cache_result_ring_buf = RingBuf::<CacheResult>::new(cache_args.buf_len);
    init_ringbuf_element!(cache_result_ring_buf, CacheResult, {
        id => cache_args.id,
        owner_ring => &mut cache_result_ring_buf as *mut RingBuf<CacheResult>,
    });


    let rx_result_list = Array::<&mut RxResult>::new(cache_args.batch_count);
    let mut next_pipeline_core = 0;
    let mut tss_key_store = KeyStore {
        key: Array::new(cache_args.hdr_max_len),
        key_len: 0,
    };
    loop {
        let rx_result_dequeue_count = cache_args.ring.dequeue_burst::<RxResult>(&rx_result_list, cache_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            let cache_result = cache_result_ring_buf.malloc();
            let rx_result = rx_result_list.get(i);
            let hash_calc_result = unsafe { &mut *rx_result.hash_calc_result };

            cache_result.is_cache_hit = false;
            cache_result.rx_result = (*rx_result) as *mut RxResult;

            if hash_calc_result.is_lbf_hit {
                // l2 cache
                if cache_args.l2_cache[hash_calc_result.l2_hash as usize].cmp_ptr_key(hash_calc_result.l2_key.as_ptr(), hash_calc_result.l2_key_len as isize) {
                    cache_result.cache_data = cache_args.l2_cache[hash_calc_result.l2_hash as usize].data.clone();
                    cache_result.is_cache_hit = true;

                    cache_args.pipeline_ring_list[next_pipeline_core].enqueue(cache_result);
                    next_pipeline_core = next_core(next_pipeline_core, cache_args.pipeline_ring_list.len());
                    continue;
                }
            }


            // l3 cache (tss)
            let cache_data = cache_args.l3_cache.search(rx_result.raw_pkt, &mut tss_key_store);
            match cache_data {
                Some(cache_data) => {
                    cache_result.cache_data = cache_data;
                    cache_result.is_cache_hit = true;
                },
                None => {},
            }

            cache_args.pipeline_ring_list[next_pipeline_core].enqueue(cache_result);
            next_pipeline_core = next_core(next_pipeline_core, cache_args.pipeline_ring_list.len());
            continue;
        }


        if false {
            return 0;
        }
    }
    // 0
}
