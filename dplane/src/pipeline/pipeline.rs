use crate::core::memory::array::Array;
// use crate::core::memory::ring::RingBuf;
use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::table::{Table, ActionSet};

// runtime native api
use crate::pipeline::runtime_native_api::debug;
use crate::pipeline::runtime_native_api::search_table;
use crate::pipeline::runtime_native_api::read_pkt;
use crate::pipeline::runtime_native_api::write_pkt;
use crate::pipeline::runtime_native_api::get_metadata;
use crate::pipeline::runtime_native_api::set_metadata;
use crate::pipeline::runtime_native_api::get_action_id;
use crate::pipeline::runtime_native_api::get_action_data;
use crate::pipeline::runtime_native_api::to_controller;
use crate::pipeline::runtime_native_api::drop;


pub struct Pipeline<'a> {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
    table_list: &'a Array<Table<'a>>,
}


impl<'a> Pipeline<'a> {
    pub fn new(wasm: &[u8], table_list: &'a Array<Table<'a>>) -> Self {
        let runtime = runtime::new_runtime!(
            wasm,
            {
                "debug" => debug,
                "search_table" => search_table,
                "read_pkt" => read_pkt,
                "write_pkt" => write_pkt,
                "get_metadata" => get_metadata,
                "set_metadata" => set_metadata,
                "get_action_id" => get_action_id,
                "get_action_data" => get_action_data,
                "to_controller" => to_controller,
                "drop" => drop,
            }
        );

        let mut runtime_args = runtime::new_runtime_args!(3);
        runtime::set_runtime_arg_i64!(runtime_args, 0, table_list as *const Array<Table> as i64);

        Pipeline {
            runtime,
            runtime_args,
            table_list,
        }
    }

    pub fn run_pipeline(&mut self, pkt: *mut u8, parse_result: &mut ParseResult) {
        runtime::set_runtime_arg_i64!(self.runtime_args, 1, pkt as i64);
        runtime::set_runtime_arg_i64!(self.runtime_args, 2, parse_result as *mut ParseResult as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }

    // pub fn run_cache_pipeline(&self, pkt: &[u8], parse_result: &ParseResult) {

    // }
}
