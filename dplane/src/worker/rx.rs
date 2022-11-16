use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::ring::Ring;
use crate::core::memory::ring::RingBuf;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::core::runtime::wasm::runtime::RuntimeArgs;
// use crate::core::runtime::wasm::runtime::new_runtime_args;
use crate::parser::header::Header;
use crate::parser::parser::Parser;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheElement;
use crate::cache::hash::l1_hash_function_murmurhash3;
use crate::cache::hash::l2_hash_function_murmurhash3;


#[repr(C)]
pub struct RxArgs {
    pub id: usize,
    pub name: String,
    pub parser: Parser,

    pub batch_count: usize,
    pub pktbuf_len: usize,

    pub l1_cache: Array<CacheElement>,
    pub lbf: Array<u8>,
    pub l2_key_max_len: u8,

    pub header_list: Array<Header>,
    pub cache_ring_list: Array<Ring>,
    pub pipeline_ring_list: Array<Ring>,
}

pub struct RxResult<'a> {
    pub owner_ring: &'a RingBuf<RxResult<'a>>,

    pub id: usize,
    pub pktbuf: &'a mut PktBuf,
    pub raw_pkt: *mut u8,
    pub parse_result: ParseResult,
    pub runtime_args: &'a RuntimeArgs,

    pub l2_key: Array<u8>,
    pub l2_key_len: u8,
    pub l2_hash: u16,
    pub is_lbf_check: bool,
}

impl<'a> RxResult<'a> {
    pub fn free(&mut self) {
        self.pktbuf.free();
        self.owner_ring.free(self);
    }
}


pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    println!("Start Rx Core");
    let rx_args = unsafe { &mut *transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    let mut pktbuf_list = Array::<PktBuf>::new(rx_args.pktbuf_len);

    let rx_result_ring_buf = RingBuf::new(rx_args.pktbuf_len);
    {
        let rx_result_array = Array::<&mut RxResult>::new(rx_args.pktbuf_len);
        rx_result_ring_buf.malloc_bulk(rx_result_array.as_slice(), rx_result_array.len());

        for (i, rx_result) in rx_result_array.as_slice().iter_mut().enumerate() {
            rx_result.id = rx_args.id;
            rx_result.owner_ring = &rx_result_ring_buf;
            rx_result.pktbuf = pktbuf_list.get(i);
            // init struct ParseResult 
            rx_result.parse_result.header_list = Array::new(rx_args.header_list.len());
            rx_result.l2_key = Array::new(rx_args.l2_key_max_len as usize);
        }

        rx_result_ring_buf.free_bulk(rx_result_array.as_slice(), rx_result_ring_buf.len());
        rx_result_array.free();
    }


    let interface = Interface::new(&rx_args.name);
    let mut next_pipeline_core = 0;
    let mut next_cache_core = 0;
    let mut next_pktbuf_index = 0;
    let rx_result_ptrs_for_reset = Array::<&mut RxResult>::new(rx_args.batch_count);
    // let mut count = 0;
    loop {
        let pkt_count = interface.rx(&mut pktbuf_list[next_pktbuf_index], rx_args.batch_count);
        for i in 0..pkt_count as usize {
            // count += 1;
            // println!("count {}, pkt {}", count, pkt_count);
            // println!("malloc");
            let rx_result = rx_result_ring_buf.malloc();
            // println!("malloc ok");
            let pktbuf = &rx_result.pktbuf;
            println!("test5 {} {}", i, next_pktbuf_index);
            let (pkt, pkt_len) = pktbuf.get_raw_pkt();
            rx_result.raw_pkt = pkt;
            if  !rx_args.parser.parse(pkt, pkt_len, &mut rx_result.parse_result) {
                continue;
            }


            // search cache
            let seed = 417;

            // l1_cache
            let l1_hash = l1_hash_function_murmurhash3(pkt, rx_result.parse_result.hdr_size, seed);
            if rx_args.l1_cache[l1_hash as usize].cmp_ptr_key(pkt, rx_result.parse_result.hdr_size as isize) {
                rx_result.runtime_args = &rx_args.l1_cache[l1_hash as usize].runtime_args;
                rx_args.pipeline_ring_list[next_pipeline_core].enqueue(rx_result);

                next_pipeline_core += 1;
                if next_pipeline_core == rx_args.pipeline_ring_list.len() {
                    next_pipeline_core = 0;
                }
                // next_pipeline_core &= rx_args.pipeline_ring_list.len();
                continue;
            }

            // lbf
            let parsed_header_list = &rx_result.parse_result.header_list;
            let mut l2_key_next_offset = 0;
            for j in 0..parsed_header_list.len() {
                if !parsed_header_list[j].is_valid {
                    continue;
                }

                let used_fields = &rx_args.header_list[j].used_fields;
                let parse_fields = &rx_args.header_list[j].parse_fields;

                for k in 0..used_fields.len() {
                    l2_key_next_offset += unsafe { used_fields[k].copy_ptr_value(parsed_header_list[j].offset as isize, pkt, rx_result.l2_key.as_ptr().offset(l2_key_next_offset)) };
                }

                for k in 0..parse_fields.len() {
                    l2_key_next_offset += unsafe { parse_fields[k].copy_ptr_value(parsed_header_list[j].offset as isize, pkt, rx_result.l2_key.as_ptr().offset(l2_key_next_offset)) };
                }

            }
            let l2_hash = l2_hash_function_murmurhash3(rx_result.l2_key.as_ptr(), rx_result.l2_key_len as usize, seed);
            rx_result.l2_hash = l2_hash;
            let core_flag = rx_args.lbf[l2_hash as usize];

            let mut is_find = false;
            for i in next_cache_core..rx_args.cache_ring_list.len() {
                if ((core_flag & (1 << i)) >> i) == 1 {
                    rx_result.is_lbf_check = true;
                    rx_args.cache_ring_list[i].enqueue(rx_result);

                    next_cache_core += 1;
                    if next_cache_core == rx_args.cache_ring_list.len() {
                        next_cache_core = 0;
                    }

                    is_find = true;
                    break;
                }

            }

            if is_find {
                continue;
            }

            for i in 0..next_cache_core {
                if ((core_flag & (1 << i)) >> i) == 1 {
                    rx_result.is_lbf_check = true;
                    rx_args.cache_ring_list[i].enqueue(rx_result);

                    next_cache_core += 1;
                    if next_cache_core == rx_args.cache_ring_list.len() {
                        next_cache_core = 0;
                    }

                    is_find = true;
                    break;
                }
            }

            if is_find {
                continue;
            }

            rx_args.cache_ring_list[next_cache_core].enqueue(rx_result);
            next_cache_core += 1;
            if next_cache_core == rx_args.cache_ring_list.len() {
                next_cache_core = 0;
            }
        }

        next_pktbuf_index += pkt_count as usize;
        if (next_pktbuf_index + rx_args.batch_count) >= pktbuf_list.len() {
            let next_pktbuf_free_space = pktbuf_list.len() - next_pktbuf_index;
            // println!("reset buf index {}, {}, {}", count, next_pktbuf_index, next_pktbuf_free_space);
            rx_result_ring_buf.malloc_bulk(rx_result_ptrs_for_reset.as_slice(), next_pktbuf_free_space);
            rx_result_ring_buf.free_bulk(rx_result_ptrs_for_reset.as_slice(), next_pktbuf_free_space);
            next_pktbuf_index = 0;
        }
    }
    0
}
