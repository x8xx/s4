use crate::core::memory::ring;
use crate::core::runtime::wasm::runtime;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;


pub struct Parser<'a> {
    runtime: runtime::Runtime<'a>,
    runtime_args: runtime::RuntimeArgs,
    buf: ring::RingBuf<ParseResult>,
}

impl<'a> Parser<'a> {
    pub fn new(wasm: &[u8], buf_len: usize, hdr_list_len: usize) -> Self {
        let runtime_args = runtime::new_runtime_args!(2);
        let runtime = runtime::new_runtime!(
            wasm,
            "parse".to_string(),
            runtime_args,
            read_pkt,
            wasm_native_func_read_pkt,
            extract_hdr,
            wasm_native_func_extract_hdr,
            set_hdr_len,
            wasm_native_func_set_hdr_len
        );

        let buf = ring::RingBuf::new(buf_len);
        let mut parse_result_ptrs: Array<&ParseResult> = Array::new(buf_len);
        buf.malloc_bulk(parse_result_ptrs, buf_len);
        for parse_result in parse_result_ptrs {
            parse_result.parse_result_of_header = Array::new(hdr_list_len);
        }
        parse_result_ptrs.free();

        Parser {
            runtime,
            runtime_args,
            buf,
        }
    }

    pub fn parse(&self, pkt: &[u8]) -> Option<&ParseResult> {
        let parse_result = self.buf.malloc();
        runtime::set_runtime_arg_i64!(self.runtime, 0, pkt.as_ptr() as i64);
        runtime::set_runtime_arg_i32!(self.runtime, 1, pkt.len() as i32);
        runtime::set_runtime_arg_i64!(self.runtime, 2, parse_result as *mut ParseResult as i64);
        let is_accept = runtime::call_runtime_i32!(self.runtime) as bool ;
        if is_accept {
            Some(parse_result)
        } else {
            self.buf.free(parse_result);
            None
        }
    }
}

pub fn wasm_native_func_read_pkt(pkt_id: i64, offset: i32) -> i32 {
    let pkt_ptr = pkt_id as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}

pub fn wasm_native_func_extract_hdr(parse_result_id: i64, hdr_id: i64, offset: i32) {
    let parse_result = parse_result_id as  *mut ParseResult as &mut ParseResult;
    parse_result.parse_result_of_header_list[hdr_id].is_valid = true;
    parse_result.parse_result_of_header_list[hdr_id].offset = offset as u16;
}

pub fn wasm_native_func_set_hdr_len(parse_result_id: i64, hdr_len: usize) {
    let parse_result = parse_result_id as  *mut ParseResult as &mut ParseResult;
    parse_result.hdr_len = hdr_len;
}

// pub fn wasm_native_func_set_parse_result(parse_result_id: i64, hdr_len: i32, parsed_hdr_index: i32)  {
//     let parse_result_ptr = parse_result_id as  *mut ParseResult;
//     unsafe {
//         (*parse_result_ptr).hdr_len = hdr_len as usize;
//         (*parse_result_ptr).parsed_hdr_index = parsed_hdr_index as usize;
//     }
// }
