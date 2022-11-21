use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::parser::runtime_native_api::ParserArgs;
use crate::parser::runtime_native_api::get_pkt_len;
use crate::parser::runtime_native_api::read_pkt;
use crate::parser::runtime_native_api::extract_hdr;


pub struct Parser {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
}

impl Parser {
    pub fn new(wasm: &[u8]) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            {
                "s4_sys_get_pkt_len" => get_pkt_len,
                "s4_sys_read_pkt" => read_pkt,
                "s4_sys_extract_hdr" => extract_hdr,
            }
        );

        let runtime_args = runtime::new_runtime_args!(1);

        Parser {
            runtime,
            runtime_args,
        }
    }

    pub fn parse(&mut self, pkt: *mut u8, pkt_len: usize, parse_result: &mut ParseResult) -> bool {
        let parser_args = ParserArgs {
            pkt,
            pkt_len,
            parse_result,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &parser_args as *const ParserArgs as i64);
        // runtime::set_runtime_arg_i64!(self.runtime_args, 0, pkt as i64);
        // runtime::set_runtime_arg_i32!(self.runtime_args, 1, pkt_len as i32);
        // runtime::set_runtime_arg_i64!(self.runtime_args, 2, parse_result as *mut ParseResult as i64);

        let is_accept = runtime::call_runtime_i32!(self.runtime, "parse", self.runtime_args);
        if is_accept == 1 {
            true
        } else {
            false
        }
    }
}
