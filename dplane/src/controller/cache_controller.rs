use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::pipeline::table::Table;
use crate::worker::pipeline::NewCacheElement;


pub fn create_new_cache(ring: Ring, table_list: Array<RwLock<Table>>) {
    let new_cache_list = Array::<&mut NewCacheElement>::new(32);
    loop {
        let new_cache_dequeue_count = ring.dequeue_burst::<NewCacheElement>(&new_cache_list, 32);
        for i in 0..new_cache_dequeue_count {
            let new_cache = new_cache_list.get(i);


            // unsafe { (*new_cache.hash_calc_result as *mut worker::rx::HashCalcResult).free(); }
            new_cache.free();
        }
    }
}
