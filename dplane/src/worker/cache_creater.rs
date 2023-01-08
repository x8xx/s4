use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null;
use std::sync::RwLock;
use crate::core::logger::log::*;
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
use crate::worker::rx::PktAnalysisResult;


#[repr(C)]
pub struct CacheCreaterArgs {
    pub ring: Ring,
    pub header_list: Array<Header>,
    pub table_list: Array<RwLock<Table>>,
    pub l1_cache_list: Array<Array<RwLock<CacheElement>>>,
    pub lbf_list: Array<Array<RwLock<u64>>>,
    pub l2_cache_list: Array<Array<Array<RwLock<CacheElement>>>>,
    // l3_cache: L3Cache
}


pub extern "C" fn start_cache_creater(args_ptr: *mut c_void) -> i32 {
    let args = unsafe { &mut *transmute::<*mut c_void, *mut CacheCreaterArgs>(args_ptr) };
    let CacheCreaterArgs { ring, header_list, table_list, l1_cache_list, lbf_list, l2_cache_list } = args;


    let pkt_analysis_result_list = Array::<&mut PktAnalysisResult>::new(32);
    loop {
        let dequeue_count = ring.dequeue_burst::<PktAnalysisResult>(&pkt_analysis_result_list, 32);
        for i in 0..dequeue_count {
            debug_log!("CacheCreater create cache...");
            let pkt_analysis_result = pkt_analysis_result_list.get(i);
            // let pkt_analysis_result = unsafe { &mut *pkt_analysis_result.pkt_analysis_result };

            // L1 Cache
            debug_log!("CacheCreater create L1 Cache");
            {
                let mut l1_cache = l1_cache_list[pkt_analysis_result.rx_id][pkt_analysis_result.l1_hash as usize].write().unwrap();
                l1_cache.key_len = pkt_analysis_result.l1_key_len as isize;
                pkt_analysis_result.l1_key.deepcopy(&mut l1_cache.key);
                pkt_analysis_result.cache_data.deepcopy(&mut l1_cache.data);
            }
            debug_log!("CacheCreater DONE create L1 Cache hash:{}", pkt_analysis_result.l1_hash);


            // L2 Cache
            debug_log!("CacheCreater create L2 Cache");
            {
                let mut l2_cache = l2_cache_list[pkt_analysis_result.rx_id][pkt_analysis_result.cache_id][pkt_analysis_result.l2_hash as usize].write().unwrap();
                // println!("L2 Hash: {}", pkt_analysis_result.l2_hash);
                l2_cache.key_len = pkt_analysis_result.l2_key_len as isize;
                pkt_analysis_result.l2_key.deepcopy(&mut l2_cache.key);
                pkt_analysis_result.cache_data.deepcopy(&mut l2_cache.data);
            }
            debug_log!("CacheCreater DONE create L2 Cache id:{} hash:{}", pkt_analysis_result.cache_id,  pkt_analysis_result.l2_hash);


            // LBF
            debug_log!("CacheCreater flag up in LBF");
            {
                let mut core_flag = lbf_list[pkt_analysis_result.rx_id][pkt_analysis_result.l2_hash as usize].write().unwrap();
                *core_flag |= 1 << pkt_analysis_result.cache_id;
            }
            debug_log!("CacheCreater DONE flag up in LBF");


            // L3 Cache
            {
                // let parsed_header = unsafe { (*(pkt_analysis_result.parse_result)).header_list };
                // let entries = pkt_analysis_result.cache_data;
                // let tuple_fields = Vec::new();

                // // j = table_id
                // for j in 0..entries.len() {
                //     // entry null check
                //     // entry != null -> used table
                //     let entry = unsafe { 
                //         if entries[j] == null() {
                //             continue
                //         }
                //         &*entries[j]
                //     };
                    
                //     let table = table_list[j].read().unwrap();
                    
                //     // k = key_id
                //     for k in 0..table.keys.len() {
                //         // Type (table::MatchField(HeaderID, FieldID), MatchKind(Exact, Lpm))
                //         let match_field = table.keys[k];

                //         let field = header_list[match_field.0.0 as usize].fields[match_field.0.1 as usize].clone();
                //         field.start_byte_pos += parsed_header[j].offset as usize;
                //         field.end_byte_pos += parsed_header[j].offset as usize;

                //         // value
                //         match match_field.1 {
                //             table::MatchKind::Exact => {
                //                 let start = Array::new(0);
                //                 let end = Array::new(0);
                //                 tuple_fields.push((tss::MatchKind::Exact(start, end), field));
                //             },
                //             table::MatchKind::Lpm => {
                //                 match entry.values[k].value {
                //                     Some(value) => {
                //                         let len = value.len();
                //                         let end_byte_mask = entry.values[k].prefix_mask;
                //                         tuple_fields.push((tss::MatchKind::Lpm(len, end_byte_mask), field));
                //                     },
                //                     // any
                //                     None => {},
                //                 }
                //             },
                //         }
                //     }

                //     // tuple hash
                // }

            }


            // pkt_analysis_result.free();
            pkt_analysis_result.free();

            debug_log!("CacheCreater cosmplete insert to cache");
        }


        if false {
            return 0;
        }
    }
}
