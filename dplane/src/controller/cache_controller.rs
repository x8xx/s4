use std::ptr::null;
use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::parser::header::Header;
use crate::parser::header::Field;
use crate::cache::cache::CacheElement;
use crate::cache::tss::L3Cache;
use crate::cache::tss;
use crate::cache::tss::TupleSpace;
// use crate::cache::cache::CacheData;
use crate::pipeline::table;
use crate::pipeline::table::Table;
use crate::worker::pipeline::NewCacheElement;


pub fn create_new_cache(ring: Ring, 
                        header_list: Array<Header>,
                        table_list: Array<RwLock<Table>>,
                        l1_cache_list: Array<Array<RwLock<CacheElement>>>,
                        lbf_list: Array<Array<u64>>,
                        l2_cache_list: Array<Array<Array<RwLock<CacheElement>>>>,
                        l3_cache: L3Cache) {
    let new_cache_list = Array::<&mut NewCacheElement>::new(32);
    loop {
        let new_cache_dequeue_count = ring.dequeue_burst::<NewCacheElement>(&new_cache_list, 32);
        for i in 0..new_cache_dequeue_count {
            // println!("check 20");
            let new_cache = new_cache_list.get(i);
            let hash_calc_result = unsafe { &mut *new_cache.hash_calc_result };

            // L1 Cache
            {
                let mut l1_cache = l1_cache_list[new_cache.rx_id][hash_calc_result.l1_hash as usize].write().unwrap();
                // println!("L1 Hash: {}", hash_calc_result.l1_hash);
                l1_cache.key_len = new_cache.l1_key_len as isize;
                new_cache.l1_key.deepcopy(&mut l1_cache.key);
                new_cache.cache_data.deepcopy(&mut l1_cache.data);
            }


            // L2 Cache
            {
                let mut l2_cache = l2_cache_list[new_cache.rx_id][new_cache.cache_id][hash_calc_result.l2_hash as usize].write().unwrap();
                // println!("L2 Hash: {}", hash_calc_result.l2_hash);
                l2_cache.key_len = hash_calc_result.l2_key_len as isize;
                hash_calc_result.l2_key.deepcopy(&mut l2_cache.key);
                new_cache.cache_data.deepcopy(&mut l2_cache.data);
            }


            // L3 Cache
            // {
            //     let parsed_header = unsafe { (*(new_cache.parse_result)).header_list };
            //     let entries= new_cache.cache_data;
            //     let tuple_fields = Vec::new();

            //     // j = table_id
            //     for j in 0..entries.len() {
            //         // entry null check
            //         let entry = unsafe { 
            //             if entries[j] == null() {
            //                 continue
            //             }
            //             &*entries[j]
            //         };
                    
            //         let table = table_list[j].read().unwrap();
                    
            //         // k = entry_id
            //         for k in 0..table.keys.len() {
            //             // (table::MatchField(HeaderID, FieldID), MatchKind(Exact, Lpm))
            //             let match_field = table.keys[k];

            //             let field = header_list[table.keys[k].0.0 as usize].fields[table.keys[k].0.1 as usize].clone();
            //             field.start_byte_pos += parsed_header[j].offset as usize;
            //             field.end_byte_pos += parsed_header[j].offset as usize;

            //             // value


            //             match match_field.1 {
            //                 table::MatchKind::Exact => {
            //                     tuple_fields.push((tss::MatchKind::Exact(), field));
            //                 },
            //                 table::MatchKind::Lpm => {
            //                     tuple_fields.push((tss::MatchKind::Lpm, field));
            //                 },
            //             }
            //         }
            //     }

            // }


            hash_calc_result.free();
            new_cache.free();
        }
    }
}
