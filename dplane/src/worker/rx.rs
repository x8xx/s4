use std::ffi::c_void;
use std::mem::transmute;
use std::sync::RwLock;
use crate::core::logger::log::log;
use crate::core::logger::log::debug_log;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::ring::{init_ringbuf_element, malloc_ringbuf_all_element, free_ringbuf_all_element};
// use crate::core::memory::heap::Heap;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::parser::header::Header;
use crate::parser::parser::Parser;
use crate::parser::parse_result::ParseResult;
use crate::parser::parse_result::Metadata;
use crate::cache::cache::CacheElement;
use crate::cache::cache::CacheData;
use crate::cache::hash::l1_hash_function_murmurhash3;
use crate::cache::hash::l2_hash_function_murmurhash3;
// use crate::cache::hash::l2_hash_function_murmurhash3_2;


#[repr(C)]
pub struct RxArgs {
    pub id: usize,
    pub interface: Interface,
    pub parser: Parser,

    pub batch_count: usize,
    pub pktbuf_size: usize,
    pub table_list_len: usize,
    pub header_max_size: usize,

    // cache
    pub l1_hash_seed: u32,
    pub l2_hash_seed: u32,
    pub l1_cache: Array<RwLock<CacheElement>>,
    pub lbf: Array<RwLock<u64>>,
    pub l2_key_max_len: u8,

    // list
    pub header_list: Array<Header>,
    pub cache_ring_list: Array<Ring>,
    pub pipeline_ring_list: Array<Ring>,
}


pub struct Pkt<'a> {
    pub owner_ring: *mut RingBuf<Pkt<'a>>,

    // pkt mbuf
    pub pktbuf: PktBuf,
    pub raw_pkt: *mut u8,
    pub len: usize,

    pub pkt_analysis_result: &'a mut PktAnalysisResult<'a>,
}


pub struct PktAnalysisResult<'a> {
    pub owner_ring: *mut RingBuf<PktAnalysisResult<'a>>,

    // core id
    pub rx_id: usize,
    pub cache_id: usize,


    // parse result
    pub parse_result: ParseResult,


    // cache
    pub cache_data: CacheData,
    pub is_cache_hit: bool,


    pub l1_key: Array<u8>,
    pub l1_key_len: usize,
    pub l1_hash: u16,

    pub l2_key: Array<u8>,
    pub l2_key_len: u8,
    pub l2_hash: u16,

    pub is_lbf_hit: bool,
}


impl<'a> Pkt<'a> {
    pub fn free(&mut self) {
        unsafe {
            debug_log!("Free Pkt {:x}", self as *mut Pkt as u64);
            (*self.owner_ring).free(self);
        }
    }
}


impl<'a> PktAnalysisResult<'a> {
    pub fn free(&mut self) {
        unsafe {
            debug_log!("Free PktAnalysisResult {:x}", self as *mut PktAnalysisResult as u64);
            (*self.owner_ring).free(self);
        }
    }
}


fn select_cache_core(core_flag: u64, core_len: usize, start_pos: usize) -> usize {
    for i in start_pos..core_len {
        if ((core_flag & (1 << i)) >> i) == 1 {
            return i;
        }
    }
    for i in 0..start_pos {
        if ((core_flag & (1 << i)) >> i) == 1 {
            return i;
        }
    }
    return core_len;
}


/**
 * Rx Worker Main
 * next -> cache || pipeline
 */
pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };
    log!("Init Rx{} Core - Port {} Queue {}", rx_args.id, rx_args.interface.port, rx_args.interface.queue);


    // let mut heap = Heap::new(rx_args.pktbuf_size * (
    //             1 + rx_args.header_list.len() + rx_args.l2_key_max_len as usize + rx_args.header_max_size + rx_args.table_list_len
    //         )
    //     );

    // init  ringbuf (Pkt)
    debug_log!("Rx{} init pkt_ring_buf", rx_args.id);
    let mut pkt_ring_buf = RingBuf::<Pkt>::new(rx_args.pktbuf_size);
    init_ringbuf_element!(pkt_ring_buf, Pkt, {
        owner_ring => &mut pkt_ring_buf as *mut RingBuf<Pkt>,
    });
    debug_log!("Rx{} done init pkt_ring_buf", rx_args.id);


    // init ringbuf (PktAnalysisResult)
    debug_log!("Rx{} init pkt_analysis_result_ring_buf", rx_args.id);
    let mut pkt_analysis_result_ring_buf = RingBuf::<PktAnalysisResult>::new(rx_args.pktbuf_size);
    {
        let pkt_analysis_result_array = malloc_ringbuf_all_element!(pkt_analysis_result_ring_buf, PktAnalysisResult);
        for (_, pkt_analysis_result) in pkt_analysis_result_array.as_slice().iter_mut().enumerate() {
            pkt_analysis_result.owner_ring = &mut pkt_analysis_result_ring_buf as *mut RingBuf<PktAnalysisResult>;


            pkt_analysis_result.rx_id = rx_args.id;


            pkt_analysis_result.parse_result.metadata = Array::new(1); 
            pkt_analysis_result.parse_result.metadata[Metadata::InPort as usize] = rx_args.id as u32 + 1;
            pkt_analysis_result.parse_result.header_list = Array::new(rx_args.header_list.len());


            pkt_analysis_result.l1_key = Array::new(rx_args.header_max_size);
            pkt_analysis_result.cache_data = Array::new(rx_args.table_list_len);
            pkt_analysis_result.l2_key = Array::new(rx_args.l2_key_max_len as usize);
        }
        free_ringbuf_all_element!(pkt_analysis_result_ring_buf, pkt_analysis_result_array);
    }
    debug_log!("Rx{} done init pkt_analysis_result_ring_buf", rx_args.id);



    let mut next_pipeline_core = 0;
    let mut random_cache_core_index = 0;
    let pktbuf_list = Array::<PktBuf>::new(rx_args.batch_count);

    let mut count = 0;

    log!("Start Rx{} Core - Port {} Queue {}", rx_args.id, rx_args.interface.port, rx_args.interface.queue);
    loop {
        let pkt_count = rx_args.interface.rx(&pktbuf_list, rx_args.batch_count);
        // debug_log!("pkt_count {}", pkt_count);
        for i in 0..pkt_count as usize {
            {
                count += 1;
                debug_log!("Rx{} pkt count {} {}", rx_args.id, count, i);
                // pktbuf_list.get(i).free();
                // continue;
            }
            // rx_args.interface.debug_show_info();


            debug_log!("Rx{} get raw pkt", rx_args.id);
            // pkt_analysis_result.pktbuf = pktbuf_list.get(i).clone();
            let (raw_pkt, raw_pkt_len) = pktbuf_list.get(i).get_raw_pkt();
            if raw_pkt_len == 0 {
                // debug_log!("Rx{} raw pkt was null", rx_args.id);
                // pkt_analysis_result.free();
                continue;
            }
            debug_log!("Rx{} done get raw pkt", rx_args.id);



            debug_log!("Rx{} pkt and pkt_analysis_result malloc", rx_args.id);
            let pkt = pkt_ring_buf.malloc();
            let pkt_analysis_result = pkt_analysis_result_ring_buf.malloc();
            debug_log!("Rx{} done pkt {:x} and pkt_analysis_result {:x} malloc", rx_args.id, pkt as *mut Pkt as i64, pkt_analysis_result as *mut PktAnalysisResult as i64);


            pkt.pktbuf = pktbuf_list.get(i).clone();
            pkt.raw_pkt = raw_pkt;
            pkt.len = raw_pkt_len;


            /*
             * Run Parser
             */
            debug_log!("Rx{} start pkt parse", rx_args.id);
            pkt_analysis_result.parse_result.hdr_size = 0;
            for i in 0..pkt_analysis_result.parse_result.header_list.len() {
                pkt_analysis_result.parse_result.header_list[i].is_valid = false;
            }
            if !rx_args.parser.parse(raw_pkt, raw_pkt_len, &mut pkt_analysis_result.parse_result) {
                debug_log!("Rx{} failed pkt parse", rx_args.id);
                pkt_analysis_result.free();
                pkt.pktbuf.free();
                pkt.free();
                continue;
            }
            debug_log!("Rx{} success pkt parse", rx_args.id);



            /*
             * L1 Cache
             */
            pkt_analysis_result.is_cache_hit = false;
            // pkt_analysis_result.hash_calc_result = None;

            debug_log!("Rx{} check L1 Cache", rx_args.id);
            let l1_hash = l1_hash_function_murmurhash3(raw_pkt, pkt_analysis_result.parse_result.hdr_size, rx_args.l1_hash_seed);
            debug_log!("Rx{} check L1 Cache l1_hash: {}", rx_args.id, l1_hash);
            {
                let cache_element = rx_args.l1_cache[l1_hash as usize].read().unwrap();
                if cache_element.cmp_ptr_key(raw_pkt, pkt_analysis_result.parse_result.hdr_size as isize) {
                    debug_log!("Rx{} Hit L1 Cache", rx_args.id);
                    debug_log!("Rx{} debug1", rx_args.id);
                    pkt_analysis_result.cache_data = cache_element.data.clone();
                    pkt_analysis_result.is_cache_hit = true;
                    debug_log!("Rx{} debug2", rx_args.id);
                    pkt.pkt_analysis_result = pkt_analysis_result;

                    if rx_args.pipeline_ring_list[next_pipeline_core].enqueue(pkt) < 0 {
                        debug_log!("Rx{} failed enqueue to Pipeline Core {}", rx_args.id, next_pipeline_core);
                        // pkt_analysis_result.pktbuf.free();
                        pkt.pkt_analysis_result.free();
                        pkt.pktbuf.free();
                        pkt.free();
                        continue;
                    }
                    debug_log!("Rx{} enqueue to Pipeline Core {}", rx_args.id, next_pipeline_core);

                    next_pipeline_core += 1;
                    if next_pipeline_core == rx_args.pipeline_ring_list.len() {
                        next_pipeline_core = 0;
                    }
                    continue;
                }
            }
            debug_log!("Rx{} No Hit L1 Cache", rx_args.id);


            pkt_analysis_result.is_lbf_hit = false;
            pkt_analysis_result.l1_hash = l1_hash;



            /*
             * Load Balancer Filter
             */
            debug_log!("Rx{} check LBF", rx_args.id);
            debug_log!("Rx{} create L2 Key", rx_args.id);
            let parsed_header_list = &pkt_analysis_result.parse_result.header_list;

            // create l2_key
            let l2_key_ptr = pkt_analysis_result.l2_key.as_ptr();
            let mut l2_key_next_offset = 0;
            for j in 0..parsed_header_list.len() {
                if !parsed_header_list[j].is_valid {
                    continue;
                }

                let fields = &rx_args.header_list[j].l2_key_fields;

                for k in 0..fields.len() {
                    l2_key_next_offset += unsafe {
                        fields[k].copy_ptr_value(parsed_header_list[j].offset as isize, raw_pkt, l2_key_ptr.offset(l2_key_next_offset))
                    };
                }
            }
            pkt_analysis_result.l2_key_len = l2_key_next_offset as u8;
            debug_log!("Rx{} done L2 Key", rx_args.id);


            // LBF
            let l2_hash = l2_hash_function_murmurhash3(l2_key_ptr, pkt_analysis_result.l2_key_len as usize, rx_args.l2_hash_seed);
            // let l2_hash = l2_hash_function_murmurhash3_2(pkt, rx_args.l2_hash_seed, &rx_args.header_list, &pkt_analysis_result.parse_result.header_list);
            pkt_analysis_result.l2_hash = l2_hash;

            {
                let core_flag = rx_args.lbf[l2_hash as usize].read().unwrap();
                let mut cache_core_index = select_cache_core(*core_flag, rx_args.cache_ring_list.len(), random_cache_core_index);

                // no hit
                if cache_core_index == rx_args.cache_ring_list.len() {
                    debug_log!("Rx{} No Hit L2 Hash", rx_args.id);
                    pkt_analysis_result.is_lbf_hit = false;
                    cache_core_index = random_cache_core_index as usize;
                // hit
                } else {
                    debug_log!("Rx{} Hit L2 Hash", rx_args.id);
                    pkt_analysis_result.is_lbf_hit = true;
                }

                pkt.pkt_analysis_result = pkt_analysis_result;


                debug_log!("Rx{} enqueue to Cache Core {}", rx_args.id, cache_core_index);
                // pkt_analysis_result.hash_calc_result = Some(hash_calc_result);
                if rx_args.cache_ring_list[cache_core_index].enqueue(pkt) < 0 {
                    debug_log!("Rx{} failed enqueue to Cache Core {}", rx_args.id, cache_core_index);
                    pkt.pkt_analysis_result.free();
                    pkt.pktbuf.free();
                    pkt.free();
                }
                debug_log!("Rx{} complete  enqueue to Cache Core {}", rx_args.id, cache_core_index);
            }


            random_cache_core_index += 1;
            if random_cache_core_index as usize == rx_args.cache_ring_list.len() {
                random_cache_core_index = 0;
            }


            if false {
                return 0;
            }
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_select_cache_core() {
        let index = super::select_cache_core(0b0, 8, 0);
        assert_eq!(index, 8);
        let index = super::select_cache_core(0b1, 8, 0);
        assert_eq!(index, 0);
        let index = super::select_cache_core(0b10, 8, 0);
        assert_eq!(index, 1);
        let index = super::select_cache_core(0b100, 8, 0);
        assert_eq!(index, 2);
        let index = super::select_cache_core(0b1000, 8, 0);
        assert_eq!(index, 3);
        let index = super::select_cache_core(0b10000, 8, 0);
        assert_eq!(index, 4);
        let index = super::select_cache_core(0b100000, 8, 0);
        assert_eq!(index, 5);
        let index = super::select_cache_core(0b1000000, 8, 0);
        assert_eq!(index, 6);
        let index = super::select_cache_core(0b10000000, 8, 0);
        assert_eq!(index, 7);
        let index = super::select_cache_core(0b100000000, 8, 0);
        assert_eq!(index, 8);
        let index = super::select_cache_core(0b1000000000, 8, 0);
        assert_eq!(index, 8);

        let index = super::select_cache_core(0b11, 8, 1);
        assert_eq!(index, 1);
        let index = super::select_cache_core(0b0000100, 8, 4);
        assert_eq!(index, 2);
    }
}
