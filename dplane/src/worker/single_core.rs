use std::ffi::c_void;
use std::mem::transmute;
use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::parser::header::Header;
use crate::parser::parser::Parser;
use crate::parser::parse_result::ParseResult;
use crate::parser::parse_result::Metadata;
use crate::cache::hash::l1_hash_function_murmurhash3;
use crate::cache::cache::CacheElement;
use crate::cache::tss::TupleSpace;
use crate::cache::tss::KeyStore;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::output::Output;


#[repr(C)]
pub struct SingleCoreArgs<'a> {
    pub id: usize,
    pub batch_count: usize,
    pub rx_interface: Interface,

    pub header_list: Array<Header>,
    pub header_max_size: usize,

    pub parser: Parser,

    pub l1_hash_seed: u32,
    pub l1_cache: Array<RwLock<CacheElement>>,
    pub l3_cache: &'a TupleSpace<'a>,

    pub pipeline: Pipeline,

    pub tx_interfaces: Array<Interface>,
}


pub extern "C" fn start_single_core(args_ptr: *mut c_void) -> i32 {
    let args = unsafe { &mut *transmute::<*mut c_void, *mut SingleCoreArgs>(args_ptr) };


    let mut pktbuf_list = Array::<PktBuf>::new(args.batch_count);


    let mut parse_result = ParseResult {
        metadata: Array::new(1),
        hdr_size: 0,
        header_list: Array::new(args.header_list.len()),
    };
    parse_result.metadata[Metadata::InPort as usize] = args.id as u32 + 1;


    let mut cache_data = Array::new(0);
    let mut tss_key_store = KeyStore {
        key: Array::new(args.header_max_size),
        key_len: 0,
    };


    loop {
        let pkt_count = args.rx_interface.rx(&pktbuf_list, args.batch_count);

        for i in 0..pkt_count as usize {
            let mut pktbuf = pktbuf_list[i].clone();
            let (pkt, pkt_len) = pktbuf.get_raw_pkt();
            if pkt_len == 0 {
                pktbuf.free();
                continue;
            }


            parse_result.hdr_size = 0;
            for i in 0..parse_result.header_list.len() {
                parse_result.header_list[i].is_valid = false;
            }
            if !args.parser.parse(pkt, pkt_len, &mut parse_result) {
                pktbuf.free();
                continue;
            }


            let l1_hash = l1_hash_function_murmurhash3(pkt, parse_result.hdr_size, args.l1_hash_seed);
            let is_cache_hit;
            {
                let cache_element = args.l1_cache[l1_hash as usize].read().unwrap();
                if cache_element.cmp_ptr_key(pkt, parse_result.hdr_size as isize) {
                    cache_data = cache_element.data.clone();
                    is_cache_hit = true;
                } else {
                    // l3 cache (tss)
                    match args.l3_cache.search(pkt, &mut tss_key_store) {
                        Some(data) => {
                            cache_data = data;
                            is_cache_hit = true;
                        },
                        None => {
                            is_cache_hit = false;
                        },
                    }
                }
            }


            let mut output = Output::Drop;
            if is_cache_hit {
                args.pipeline.run_cache_pipeline(pkt, pkt_len,  &parse_result, &mut cache_data, &mut output);
            } else {
                // let new_cache_element = new_cache_element_ringbuf.malloc();

                // pipeline_args.pipeline.run_pipeline(*raw_pkt, *pkt_len, parse_result, &mut new_cache_element.cache_data, &mut output);

                // new_cache_element.l1_key_len = parse_result.hdr_size;
                // for i in 0..new_cache_element.l1_key_len {
                //     unsafe {
                //         new_cache_element.l1_key[i] = *raw_pkt.offset(i as isize);
                //     }
                // }
                // new_cache_element.rx_id = *rx_id;
                // new_cache_element.cache_id = *cache_id;
                // new_cache_element.hash_calc_result = *hash_calc_result as *const HashCalcResult as *mut HashCalcResult;

                // debug_log!("Pipeline{} enqueue to cache_creater_ring", pipeline_args.id);
                // // to cache_creater (main core)
                // pipeline_args.cache_creater_ring.enqueue(new_cache_element);
            }



            match output {
                Output::Port(port_num) => {
                    // args.tx_interfaces[port_num as usize].tx(&mut pktbuf, 1);
                },
                Output::All => {
                    // TODO
                },
                Output::Controller => {
                    // debug_log!("Pipeline enqueue to Tx0 (CPU)");
                    // if tx_ring_list[0][0].enqueue(pktbuf) < 0 {
                    //     return false;
                    // }
                },
                Output::Drop => {
                },
            }
        } 

        if false {
            return 0;
        }
    }
}
