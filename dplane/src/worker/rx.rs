use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;

use crate::core::network::pktbuf::PktBuf;
use crate::core::network::interface::Interface;
use crate::parser::parser::Parser;

#[repr(C)]
pub struct RxArgs<'a> {
    pub name: String,
    pub parser: &'a Parser<'a>,
}

pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    println!("Start Rx Core");
    let rx_args = unsafe { &*transmute::<*mut c_void, *mut RxArgs>(rx_args_ptr) };

    let interface = Interface::new(&rx_args.name);
    let pktbuf_len = 32;
    let pktbuf = PktBuf::new(pktbuf_len);
    
    loop {
        interface.rx(&pktbuf);
        for i in 0..pktbuf_len {
            let (pkt, pkt_len) = pktbuf.get_raw_pkt(i);
            // let parse_result = rx_args.parser.parse(pkt, pkt_len);
            // match parse_result {
            //     None => continue,
            //     _ => {},
            // }
        }
    }
    0
}

//pub extern "C" fn rx_start(rx_start_args_ptr: *mut c_void) -> i32 {
//    println!("😟Loading Rx Core");
//    let rx_start_args = unsafe {&*transmute::<*mut c_void, *mut RxStartArgs>(rx_start_args_ptr)};
//    let if_name = &rx_start_args.if_name;

//    let hdrs = rx_start_args.hdrs;
//    let hdrs_len = rx_start_args.hdrs_len;
//    let parser = rx_start_args.parser;

//    let l1_cache = rx_start_args.l1_cache;
//    let l1_cache_key = rx_start_args.l1_cache_key;
//    let l1_key_max_len = rx_start_args.l1_key_max_len;

//    let lb_filter = &rx_start_args.lb_filter;

//    let fib_core_rings = &rx_start_args.fib_core_rings;

//    println!("create mbuf");
//    let pktmbuf = dpdk_memory::create_pktmbuf("mbuf");
//    let port_number = dpdk_port::port_init(if_name, pktmbuf);

//    println!("create pktprocessor");
//    let pp = dpdk_eth::PktProcessor::new(port_number);


//    let packet_batch_num = 32;
//    let parsed_hdrs = dpdk_memory::malloc::<*mut (u8, *mut Header)>("rx_parsed_hdrs".to_string(), packet_batch_num);
//    for  i in 0..packet_batch_num {
//        unsafe {
//            let parsed_hdr_name = format!("rx_parsed_hdr_{}", i);
//            *parsed_hdrs.offset(i as isize) = dpdk_memory::malloc::<(u8, *mut Header)>(parsed_hdr_name, hdrs_len);
//        }

//    }

//    let mut wasm_parser_args = WasmParserArgs {
//        hdrs: hdrs as *mut Header,
//        parsed_hdr: null_mut(),
//        size: 0,
//    };
//    let wasm_parser_args_ptr = &mut wasm_parser_args as *mut WasmParserArgs;

//    let mut random_next_core = 0;
//    let mut parser_args: [wasmer::Value;3] = [wasmer::Value::I64(0), wasmer::Value::I32(0), wasmer::Value::I64(0)];


//    // tmp memory
//    let l2_cache_key_mem_ptr = dpdk_memory::malloc::<u8>("l2_key_rx_mem".to_string(), l1_key_max_len);
//    let l2_key = unsafe { from_raw_parts_mut(l2_cache_key_mem_ptr, l1_key_max_len as usize) };

//    println!("👍Rx Core Ready!");

//    println!("🚀Launch Switch Port {}", if_name);
//    loop {
//        let rx_count = pp.rx();
//        if rx_count <= 0 {
//            continue;
//        }
//        for i in 0..rx_count {
//            let pkt = pp.get_packet(i);
            
//            wasm_parser_args.size = 0;
//            unsafe {
//                wasm_parser_args.parsed_hdr = *parsed_hdrs.offset(i as isize);
//            }

//            parser_args[0] = wasmer::Value::I64(pkt.as_ptr() as i64);
//            parser_args[1] = wasmer::Value::I32(pkt.len() as i32);
//            parser_args[2] = wasmer::Value::I64(wasm_parser_args_ptr as i64);
//            let parse_result= parser.call(&parser_args);
//            let parsed_byte =  match parse_result {
//                Ok(result) => result[0].unwrap_i32() as usize,
//                Err(_) => continue,
//            };

//            if parsed_byte == 0 {
//                println!("continue");
//                continue;
//            }

//            println!("check parsed hdr {}", parsed_byte);
//            for j in 0..wasm_parser_args.size {
//                println!("base offset {}", unsafe { (*(*parsed_hdrs.offset(i as isize)).offset(j as isize)).0  });
//            }

//            let l1_key = &pkt[0..parsed_byte];
//            let l1_hash = murmurhash3::murmurhash3_x86_32(l1_key, 1) >> 16;
//            println!("l1_hash {}", l1_hash);
//            if cache::key_compare_slice_pointer(l1_key, l1_cache_key as *const u8, (l1_hash * l1_key_max_len) as isize) {
//                // next core (push ring)
//                continue;
//            }

            
//            let parsed_hdr = unsafe { *parsed_hdrs.offset(i as isize) };
//            let mut l2_key_pos = 0;
//            for j in 0..wasm_parser_args.size {
//                unsafe {
//                    let hdr_base_offset = (*parsed_hdr.offset(j)).0;
//                    let used_fields = (*(*parsed_hdr.offset(j)).1).used_fields;
//                    let used_fields_len = (*(*parsed_hdr.offset(j)).1).used_fields_len;
//                    for k in 0..used_fields_len {
//                        let field = *used_fields.offset(k);
//                        l2_key[l2_key_pos] =  pkt[hdr_base_offset as usize + field.start_byte_pos] ^ field.start_bit_mask;
//                        l2_key_pos += 1;
//                        if field.start_byte_pos != field.end_byte_pos {
//                            for l in field.start_byte_pos+1..field.end_byte_pos {
//                                l2_key[l2_key_pos] = pkt[hdr_base_offset as usize + k as usize];
//                                l2_key_pos += 1;
//                            } 
//                            l2_key[l2_key_pos] =  pkt[hdr_base_offset as usize + field.end_byte_pos] ^ field.end_bit_mask;
//                            l2_key_pos += 1;
//                        }
//                    }
//                }
//            }

//            let l2_hash = murmurhash3::murmurhash3_x86_32(l2_key, 1) >> 16;
//            println!("l2_hash {}", l2_hash);
//            let core_flag = unsafe { lb_filter.offset(l2_hash as isize) };
//            // unsafe {
//            //     let bit_count = core::arch::x86_64::_popcnt64(0xff as i64);
//            //     println!("bit count {}", bit_count);
//            //
//            // }


//            println!("chalange {:p} {}", pkt.as_ptr(), fib_core_rings[0].enqueue::<u8>(pkt.as_ptr() as *mut u8));

//        }
//        // pp.tx();
//    }

//    0
//}


// #[repr(C)]
// pub struct RxMainArgs<'a> {
//     pub core_id: u8,

//     pub if_name: &'a str,

//     pub hdrs: *const Header,
//     pub hdrs_len: u32,
//     pub parser: *const wasmer::Function,

//     pub l1_cache: *mut cache::Cache,
//     pub lb_filter: *mut u8,

//     pub fib_core_rings: &'a [dpdk_memory::Ring],
// }


// pub extern "C" fn rx_main(args_ptr: *mut c_void) -> i32 {
//     let args = unsafe {&*transmute::<*mut c_void, &RxMainArgs>(args_ptr)};
//     println!("Core{}: Loading Rx Core...", args.core_id);

//     println!("Core{}: Creating mbuf...", args.core_id);
//     let pktmbuf = dpdk_memory::create_pktmbuf("mbuf");
//     let port_number = dpdk_port::port_init(args.if_name, pktmbuf);

//     println!("create pktprocessor");
//     let pp = dpdk_eth::PktProcessor::new(port_number);

//     // let if_name = &rx_start_args.if_name;
//     // let hdrs = rx_start_args.hdrs;
//     // let hdrs_len = rx_start_args.hdrs_len;
//     // let parser = rx_start_args.parser;

//     // let l1_cache = rx_start_args.l1_cache;
//     // let l1_cache_key = rx_start_args.l1_cache_key;
//     // let l1_key_max_len = rx_start_args.l1_key_max_len;

//     // let lb_filter = &rx_start_args.lb_filter;

//     // let fib_core_rings = &rx_start_args.fib_core_rings;
//     0
// }
