use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::parser::runtime_native_api::ParserArgs;
use crate::parser::runtime_native_api::pkt_get_len;
use crate::parser::runtime_native_api::pkt_read;
use crate::parser::runtime_native_api::pkt_drop;
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
                "s4_sys_pkt_get_len" => pkt_get_len,
                "s4_sys_pkt_read" => pkt_read,
                "s4_sys_pkt_drop" => pkt_drop,
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
        let mut is_accept = true;
        let parser_args = ParserArgs {
            pkt,
            pkt_len,
            parse_result,
            is_accept: &mut is_accept
        };

        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &parser_args as *const ParserArgs as i64);
        runtime::call_runtime!(self.runtime, "parse", self.runtime_args);

        is_accept
    }
}
