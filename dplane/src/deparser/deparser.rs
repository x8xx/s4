use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;

use crate::deparser::runtime_native_api::DeparserArgs;
use crate::deparser::runtime_native_api::emit;

pub struct Deparser {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
}

impl Deparser {
    pub fn new(wasm: &[u8]) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            {
                "s4_sys_emit" => emit,
            }
        );

        let runtime_args = runtime::new_runtime_args!(1);

        Deparser {
            runtime,
            runtime_args,
        }
    }

    pub fn deparse(&mut self, pkt: *mut u8, parse_result: &mut ParseResult) {
        let mut deparser_args = DeparserArgs {
            pkt,
            parse_result: parse_result as *mut ParseResult,
        };

        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &mut deparser_args as *mut DeparserArgs as i64);
        runtime::call_runtime_i32!(self.runtime, "deparse", self.runtime_args);
    }
}
