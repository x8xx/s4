use std::sync::RwLock;

use crate::core::memory::array::Array;
// use crate::core::memory::ring::RingBuf;
use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::table::Table;
use crate::pipeline::tx_conf::TxConf;
// use crate::pipeline::table::ActionSet;

// runtime native api
use crate::pipeline::runtime_native_api::PipelineArgs;
use crate::pipeline::runtime_native_api::PipelineCacheArgs;
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


pub struct Pipeline {
    runtime: runtime::Runtime,
    runtime_args: runtime::RuntimeArgs,
    table_list: Array<RwLock<Table>>,
}


impl Pipeline {
    pub fn new(wasm: &[u8], table_list: Array<RwLock<Table>>) -> Self {
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

        let runtime_args = runtime::new_runtime_args!(1);

        Pipeline {
            runtime,
            runtime_args,
            table_list,
        }
    }


    pub fn run_pipeline(&mut self, pkt: *mut u8, parse_result: &ParseResult, new_cache_data: &mut CacheData, tx_conf: &mut TxConf) {
        let pipeline_args = PipelineArgs {
            table_list: &self.table_list,
            pkt,
            parse_result,
            new_cache_data,
            tx_conf,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_args as *const PipelineArgs as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }


    pub fn run_cache_pipeline(&mut self, pkt: *mut u8, parse_result: &ParseResult, cache_data: &CacheData, tx_conf: &mut TxConf) {
        let pipeline_cache_args = PipelineCacheArgs {
            table_list: &self.table_list,
            pkt,
            parse_result,
            cache_data,
            tx_conf,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_cache_args as *const PipelineCacheArgs as i64);
        runtime::call_runtime!(self.runtime, "run_cache_pipeline", self.runtime_args);
    }
}
