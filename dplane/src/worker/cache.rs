use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::array::Array;
use crate::core::runtime::wasm::runtime::RuntimeArgs;
use crate::cache::cache::CacheElement;
use crate::cache::tss::TupleSpace;
use crate::worker::rx::RxResult;


#[repr(C)]
pub struct CacheArgs<'a> {
    pub id: usize,
    pub ring: Ring,

    pub batch_count: usize,
    pub buf_len: usize,

    pub l2_cache: Array<CacheElement>,
    pub l3_cache: &'a TupleSpace,
    pub pipeline_ring_list: Array<Ring>,
}

pub struct CacheResult<'a> {
    pub rx_result: &'a RxResult<'a>,
    pub id: usize,
    pub runtime_args: &'a RuntimeArgs,
}


pub extern "C" fn start_cache(cache_args_ptr: *mut c_void) -> i32 {
    println!("Start Cache Core");
    let cache_args = unsafe { &mut *transmute::<*mut c_void, *mut CacheArgs>(cache_args_ptr) };

    let cache_result_ring_buf = RingBuf::<CacheResult>::new(cache_args.buf_len);
    {

    }

    let mut rx_result_list = Array::<&mut RxResult>::new(cache_args.batch_count);
    loop {
        let rx_result_dequeue_count = cache_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], cache_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            let rx_result = &mut rx_result_list[i];

            if rx_result.is_lbf_check {
                // l2 cache
                if cache_args.l2_cache[rx_result.l2_hash as usize].cmp_ptr_key(rx_result.l2_key.as_ptr(), rx_result.l2_key_len as isize) {

                }
            }

            // tss
            if cache_args.l3_cache.search() {

            }
        }

        // cache ring
    }
    0
}
