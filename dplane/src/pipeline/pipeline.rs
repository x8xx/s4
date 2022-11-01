use crate::core::memory::array::Array;
// use crate::core::memory::ring::RingBuf;
use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::table::ActionSet;
use crate::pipeline::table::wasm_native_func_search_table;

pub struct Pipeline {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
}

impl Pipeline {
    pub fn new(wasm: &[u8]) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            search_table,
            wasm_native_func_search_table
        );

        let runtime_args = runtime::new_runtime_args!(2);

        Pipeline {
            runtime,
            runtime_args,
        }
    }

    pub fn run_pipeline(&self, pkt: &[u8], parse_result: &ParseResult) -> Array<ActionSet> {

    }

    pub fn run_cache_pipeline(&self, pkt: &[u8], parse_result: &ParseResult) {

    }
}
