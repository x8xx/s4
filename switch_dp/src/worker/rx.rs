use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use crate::fib::*;
use crate::fib::header::Header;
use crate::fib::parser::WasmParserArgs;
use crate::dpdk::dpdk_memory;
use crate::dpdk::dpdk_port;
use crate::dpdk::dpdk_eth;

#[repr(C)]
pub struct RxStartArgs<'a> {
    pub if_name: &'a str,
    pub hdrs: *const Header,
    pub hdrs_len: u32,
    pub parser: &'a wasmer::Function,
    pub l1_cache: *mut u8,
    pub lb_filter: *mut u8,
    pub fib_core_rings: &'a [dpdk_memory::Ring],
}


fn next_core() -> usize {
    0
}


pub extern "C" fn rx_start(rx_start_args_ptr: *mut c_void) -> i32 {
    println!("😟Loading Rx Core");
    let rx_start_args = unsafe {&*transmute::<*mut c_void, *mut RxStartArgs>(rx_start_args_ptr)};
    let if_name = &rx_start_args.if_name;

    let hdrs = rx_start_args.hdrs;
    let hdrs_len = rx_start_args.hdrs_len;
    let parser = rx_start_args.parser;

    let l1_cache = &rx_start_args.l1_cache;
    let lb_filter = &rx_start_args.lb_filter;
    let fib_core_rings = &rx_start_args.fib_core_rings;

    println!("create mbuf");
    let pktmbuf = dpdk_memory::create_pktmbuf("mbuf");
    let port_number = dpdk_port::port_init(if_name, pktmbuf);

    println!("create pktprocessor");
    let pp = dpdk_eth::PktProcessor::new(port_number);

    let mut random_next_core = 0;

    let mut parser_args: [wasmer::Value;3] = [wasmer::Value::I64(0), wasmer::Value::I32(0), wasmer::Value::I64(0)];
    let packet_batch_num = 32;
    let parsed_hdrs = dpdk_memory::malloc::<*mut (u8, *mut Header)>("rx_parsed_hdrs".to_string(), packet_batch_num);
    for  i in 0..packet_batch_num {
        unsafe {
            let parsed_hdr_name = format!("rx_parsed_hdr_{}", i);
            *parsed_hdrs.offset(i as isize) = dpdk_memory::malloc::<(u8, *mut Header)>(parsed_hdr_name, hdrs_len);
        }

    }

    let mut wasm_parser_args = WasmParserArgs {
        hdrs: hdrs as *mut Header,
        parsed_hdr: null_mut(),
        size: 0,
    };
    let wasm_parser_args_ptr = &mut wasm_parser_args as *mut WasmParserArgs;


    println!("👍Rx Core Ready!");

    println!("🚀Launch Switch Port {}", if_name);
    loop {
        let rx_count = pp.rx();
        if rx_count <= 0 {
            continue;
        }
        for i in 0..rx_count {
            let pkt = pp.get_packet(i);
            
            wasm_parser_args.size = 0;
            unsafe {
                wasm_parser_args.parsed_hdr = *parsed_hdrs.offset(i as isize);
            }

            parser_args[0] = wasmer::Value::I64(pkt.as_ptr() as i64);
            parser_args[1] = wasmer::Value::I32(pkt.len() as i32);
            parser_args[2] = wasmer::Value::I64(wasm_parser_args_ptr as i64);
            let parse_result= parser.call(&parser_args);
            // println!("ok2");
            let parsed_byte =  match parse_result {
                Ok(result) => result[0].unwrap_i32() as usize,
                Err(_) => continue,
            };

            if parsed_byte == 0 {
                println!("continue");
                continue;
            }

            println!("check parsed hdr {}", parsed_byte);
            for j in 0..wasm_parser_args.size {
                println!("base offset {}", unsafe { (*(*parsed_hdrs.offset(i as isize)).offset(j as isize)).0  });
            }

            let l1_key = &pkt[0..parsed_byte];
            let l1_hash = murmurhash3::murmurhash3_x86_32(l1_key, 1);
            println!("l1_hash {}", l1_hash);
            // match l1_cache[l1_hash as usize].compare_key(l1_key) {
            // match l1_cache[0].compare_key(l1_key) {
            //     Some(u8) => continue,
            //     None => 0,
            // };


            continue;


            let l2_key = &pkt[0..112];
            let l2_hash = murmurhash3::murmurhash3_x86_32(l2_key, 1);
            println!("l2_hash {}", l2_hash);
            // let core_flag = lb_filter[l2_hash as usize];

            // rx_start_args.lb_filter[0] = 0xff;
            // lb_filter[0] = 0xff;
            unsafe {
                let bit_count = core::arch::x86_64::_popcnt64(0xff as i64);
                println!("bit count {}", bit_count);
            }

        }
        // pp.tx();
    }

    0
}
