use crate::core::memory::ring;
use crate::core::runtime::wasm::runtime;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;


pub struct Parser {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
    // parse_result_ringbuf: ring::RingBuf<ParseResult<'a>>,
}

impl Parser {
    pub fn new(wasm: &[u8], ringbuf_len: usize, hdr_list_len: usize) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            "read_pkt",
            wasm_native_func_read_pkt,
            "extract_hdr",
            wasm_native_func_extract_hdr,
            "set_hdr_len",
            wasm_native_func_set_hdr_len
        );

        let runtime_args = runtime::new_runtime_args!(3);
        
        // let parse_result_ringbuf = ring::RingBuf::new(ringbuf_len);
        // {
        //     let parse_result_array= Array::<&mut ParseResult>::new(ringbuf_len);
        //     let mut parse_result_slice = parse_result_array.as_slice();
        //     parse_result_ringbuf.malloc_bulk(&mut parse_result_slice, ringbuf_len);
        //     for parse_result in parse_result_slice.iter_mut() {
        //         (*parse_result).parse_result_of_header_list = Array::new(hdr_list_len);
        //     }
        //     parse_result_ringbuf.free_bulk(parse_result_slice, ringbuf_len);
        //     parse_result_array.free();
        // }

        Parser {
            runtime,
            runtime_args,
            // parse_result_ringbuf,
        }
    }

    pub fn parse(&mut self, pkt: *mut u8, pkt_len: usize, parse_result: &mut ParseResult) -> bool {
        // let parse_result = self.parse_result_ringbuf.malloc();
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, pkt as i64);
        runtime::set_runtime_arg_i32!(self.runtime_args, 1, pkt_len as i32);
        runtime::set_runtime_arg_i64!(self.runtime_args, 2, parse_result as *mut ParseResult as i64);

        let is_accept = runtime::call_runtime_i32!(self.runtime, "parse", self.runtime_args);
        if is_accept == 1 {
            // Some(parse_result)
            true
        } else {
            // self.parse_result_ringbuf.free(parse_result);
            false
            // None
        }
    }

    // pub fn parse_result_free(&'a self, parse_result: &'a mut ParseResult<'a>) {
    //     let parse_result_of_header_list = &mut parse_result.parse_result_of_header_list;
    //     for i in 0..parse_result_of_header_list.len() {
    //         parse_result_of_header_list[i].is_valid = false;
    //     }
    //     self.parse_result_ringbuf.free(parse_result);
    // }
}


pub fn wasm_native_func_read_pkt(pkt_id: i64, offset: i32) -> i32 {
    let pkt_ptr = pkt_id as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}

pub fn wasm_native_func_extract_hdr(parse_result_id: i64, hdr_id: i64, offset: i32) {
    let parse_result = unsafe { &mut *(parse_result_id as  *mut ParseResult) as &mut ParseResult };
    parse_result.parse_result_of_header_list[hdr_id as usize].is_valid = true;
    parse_result.parse_result_of_header_list[hdr_id as usize].offset = offset.try_into().unwrap();
}

pub fn wasm_native_func_set_hdr_len(parse_result_id: i64, hdr_len: i32) {
    let parse_result = unsafe { &mut *(parse_result_id as  *mut ParseResult) as &mut ParseResult };
    parse_result.hdr_len = hdr_len as usize;
}
