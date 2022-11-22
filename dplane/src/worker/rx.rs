use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::ring::{init_ringbuf_element, malloc_ringbuf_all_element, free_ringbuf_all_element};
use crate::core::memory::array::Array;
use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::parser::header::Header;
use crate::parser::parser::Parser;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheElement;
use crate::cache::cache::CacheData;
use crate::cache::hash::l1_hash_function_murmurhash3;
use crate::cache::hash::l2_hash_function_murmurhash3;


#[repr(C)]
pub struct RxArgs {
    pub id: usize,
    pub interface: Interface,
    pub parser: Parser,

    pub batch_count: usize,
    pub pktbuf_len: usize,

    // cache
    pub l1_hash_seed: u32,
    pub l2_hash_seed: u32,
    pub l1_cache: Array<CacheElement>,
    pub lbf: Array<u64>,
    pub l2_key_max_len: u8,

    // list
    pub header_list: Array<Header>,
    pub cache_ring_list: Array<Ring>,
    pub pipeline_ring_list: Array<Ring>,
}


pub struct RxResult {
    pub owner_ring: *mut RingBuf<RxResult>,

    // to cache and pipeline
    pub id: usize,
    pub pktbuf: PktBuf,
    pub raw_pkt: *mut u8,
    pub parse_result: ParseResult,
    pub cache_data: CacheData,

    // to cache core
    pub hash_calc_result: *mut HashCalcResult, 
}

impl RxResult {
    pub fn free(&mut self) {
        // self.pktbuf.free();
        unsafe {
            (*self.owner_ring).free(self);
        }
    }
}


pub struct HashCalcResult {
    pub owner_ring: *mut RingBuf<HashCalcResult>,
    pub l1_hash: u16,
    pub l2_key: Array<u8>,
    pub l2_key_len: u8,
    pub l2_hash: u16,
    pub is_lbf_hit: bool,
}

impl HashCalcResult {
    pub fn free(&mut self) {
        unsafe {
            (*self.owner_ring).free(self);
        }
    }
}


pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    println!("Start Rx Core");
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    // init ringbuf
    let mut rx_result_ring_buf = RingBuf::<RxResult>::new(rx_args.pktbuf_len);
    {
        let rx_result_array = malloc_ringbuf_all_element!(rx_result_ring_buf, RxResult);
        for (_, rx_result) in rx_result_array.as_slice().iter_mut().enumerate() {
            rx_result.id = rx_args.id;
            rx_result.owner_ring = &mut rx_result_ring_buf as *mut RingBuf<RxResult>;
            rx_result.parse_result.header_list = Array::new(rx_args.header_list.len());
        }
        free_ringbuf_all_element!(rx_result_ring_buf, rx_result_array);
    }


    // init hash_calc_result
    let mut hash_calc_result_ring_buf = RingBuf::<HashCalcResult>::new(rx_args.pktbuf_len * 2);
    init_ringbuf_element!(hash_calc_result_ring_buf, HashCalcResult, {
        owner_ring => &mut hash_calc_result_ring_buf as *mut RingBuf<HashCalcResult>,
        l2_key => Array::new(rx_args.l2_key_max_len as usize),
    });


    let mut next_pipeline_core = 0;
    let mut next_cache_core = 0;
    let mut pktbuf_list = Array::<PktBuf>::new(rx_args.batch_count);
    loop {
        let pkt_count = rx_args.interface.rx(&mut pktbuf_list[0], rx_args.batch_count);
        for i in 0..pkt_count as usize {
            let rx_result = rx_result_ring_buf.malloc();

            rx_result.pktbuf = pktbuf_list.get(i).clone();
            let (pkt, pkt_len) = rx_result.pktbuf.get_raw_pkt();
            if  !rx_args.parser.parse(pkt, pkt_len, &mut rx_result.parse_result) {
                continue;
            }

            rx_result.raw_pkt = pkt;
            rx_result.pktbuf = pktbuf_list[i].clone();
            

            // l1_cache
            let l1_hash = l1_hash_function_murmurhash3(pkt, rx_result.parse_result.hdr_size, rx_args.l1_hash_seed);
            if rx_args.l1_cache[l1_hash as usize].cmp_ptr_key(pkt, rx_result.parse_result.hdr_size as isize) {
                rx_result.cache_data= rx_args.l1_cache[l1_hash as usize].data.clone();
                rx_args.pipeline_ring_list[next_pipeline_core].enqueue(rx_result);

                next_pipeline_core += 1;
                if next_pipeline_core == rx_args.pipeline_ring_list.len() {
                    next_pipeline_core = 0;
                }
                continue;
            }


            let hash_calc_result = hash_calc_result_ring_buf.malloc();
            hash_calc_result.is_lbf_hit = false;
            hash_calc_result.l1_hash = l1_hash;


            // lbf
            let parsed_header_list = &rx_result.parse_result.header_list;
            let l2_key_ptr = hash_calc_result.l2_key.as_ptr();
            let mut l2_key_next_offset = 0;
            for j in 0..parsed_header_list.len() {
                if !parsed_header_list[j].is_valid {
                    continue;
                }

                let used_fields = &rx_args.header_list[j].used_fields;
                let parse_fields = &rx_args.header_list[j].parse_fields;

                for k in 0..used_fields.len() {
                    l2_key_next_offset += unsafe {
                        used_fields[k].copy_ptr_value(parsed_header_list[j].offset as isize, pkt, l2_key_ptr.offset(l2_key_next_offset))
                    };
                }

                for k in 0..parse_fields.len() {
                    l2_key_next_offset += unsafe {
                        parse_fields[k].copy_ptr_value(parsed_header_list[j].offset as isize, pkt, l2_key_ptr.offset(l2_key_next_offset))
                    };
                }
            }
            hash_calc_result.l2_key_len = l2_key_next_offset as u8;

            let l2_hash = l2_hash_function_murmurhash3(l2_key_ptr, hash_calc_result.l2_key_len as usize, rx_args.l2_hash_seed);
            hash_calc_result.l2_hash = l2_hash;

            let core_flag = rx_args.lbf[l2_hash as usize];
            let mut cache_core = select_cache_core(core_flag, rx_args.cache_ring_list.len(), next_cache_core);


            // no hit
            if cache_core == rx_args.cache_ring_list.len() {
                hash_calc_result.is_lbf_hit = false;
                cache_core = next_cache_core as usize;
            // hit
            } else {
                hash_calc_result.is_lbf_hit = true;
            }


            rx_result.hash_calc_result = hash_calc_result as *mut HashCalcResult;
            rx_args.cache_ring_list[cache_core].enqueue(rx_result);


            next_cache_core += 1;
            if next_cache_core as usize == rx_args.cache_ring_list.len() {
                next_cache_core = 0;
            }


            if false {
                return 0;
            }
        }
    }
    // 0
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
