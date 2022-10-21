use crate::parser::parse_result::ParseResult;
use crate::core::memory::ring;
use crate::core::runtime::wasm::runtime;


pub struct Parser {
    runtime: runtime::Runtime,
    buf: ring::RingBuf<ParseResult>,
}

impl Parser {
    pub fn new(wasm: &[u8], buf_len: usize) -> Self {
        Parser {
            runtime: runtime::new_runtime!(
                         wasm,
                         "parser".to_string(),
                         2,
                        read_pkt,
                        wasm_native_func_read_pkt,
                        set_parse_result,
                        wasm_native_func_set_parse_result
                    ),
            buf: ring::RingBuf::new(buf_len),
        }
    }

    pub fn parse(&self, pkt: &[u8]) -> &ParseResult {
        let parse_result = self.buf.one_malloc();
        runtime::set_runtime_args_i64!(self.runtime, 0, pkt.as_ptr() as i64);
        runtime::set_runtime_args_i32!(self.runtime, 1, pkt.len() as i32);
        runtime::set_runtime_args_i64!(self.runtime, 2, parse_result as *mut ParseResult as i64);
        let parse_result_ptr = runtime::call_runtime_i64!(self.runtime) as *const ParseResult ;
        parse_result
    }
}

pub fn wasm_native_func_read_pkt(pkt_id: i64, offset: i32) -> i32 {
    let pkt_ptr = pkt_id as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}

pub fn wasm_native_func_set_parse_result(parse_result_id: i64, hdr_len: i32, used_fields_index: i32)  {
    let parse_result_ptr = parse_result_id as  *mut ParseResult;
    unsafe {
        (*parse_result_ptr).hdr_len = hdr_len as usize;
        (*parse_result_ptr).used_fields_index = used_fields_index as usize;
    }
}
