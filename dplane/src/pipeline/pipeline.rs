use crate::core::memory::array::Array;
// use crate::core::memory::ring::RingBuf;
use crate::core::runtime::wasm::runtime;
use crate::pipeline::table::{Table, ActionSet};
use crate::pipeline::table::wasm_native_func_search_table;

pub struct Pipeline<'a> {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
    table_list: &'a Array<Table<'a>>,
}

impl<'a> Pipeline<'a> {
    pub fn new(wasm: &[u8], table_list: &'a Array<Table<'a>>) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            "search_table",
            wasm_native_func_search_table
        );

        let runtime_args = runtime::new_runtime_args!(2);

        Pipeline {
            runtime,
            runtime_args,
            table_list,
        }
    }

    // pub fn run_pipeline(&self, pkt: *mut u8, parse_result: &ParseResult) -> Array<ActionSet> {

    // }

    // pub fn run_cache_pipeline(&self, pkt: &[u8], parse_result: &ParseResult) {

    // }
}
