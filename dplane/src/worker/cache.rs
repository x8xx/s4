use std::ffi::c_void;
use std::mem::transmute;
use crate::worker::rx::RxResult;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::array::Array;

#[repr(C)]
pub struct CacheArgs<'a> {
    pub ring: &'a Ring,
    pub batch_count: usize,
    pub pipeline_ring_list: &'a Array<Ring>,
}


pub struct CacheResult {

}


pub extern "C" fn start_cache(cache_args_ptr: *mut c_void) -> i32 {
    println!("Start Cache Core");
    let cache_args = unsafe { &mut *transmute::<*mut c_void, *mut CacheArgs>(cache_args_ptr) };

    let mut rx_result_list = Array::<&mut RxResult>::new(cache_args.batch_count);
    loop {
        let rx_result_dequeue_count = cache_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], cache_args.batch_count);
        for i in 0..rx_result_dequeue_count {
            let rx_result = &mut rx_result_list[i];
            
        }

        // cache ring
    }
    0
}
