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
use crate::pipeline::runtime_native_api::flooding;


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
                "s4_sys_debug" => debug,
                "s4_sys_search_table" => search_table,
                "s4_sys_read_pkt" => read_pkt,
                "s4_sys_write_pkt" => write_pkt,
                "s4_sys_get_metadata" => get_metadata,
                "s4_sys_set_metadata" => set_metadata,
                "s4_sys_get_action_id" => get_action_id,
                "s4_sys_get_action_data" => get_action_data,
                "s4_sys_to_controller" => to_controller,
                "s4_sys_drop" => drop,
                "s4_sys_flooding" => flooding,
            }
        );

        let runtime_args = runtime::new_runtime_args!(1);

        Pipeline {
            runtime,
            runtime_args,
            table_list,
        }
    }


    pub fn run_pipeline(&mut self, pkt: *mut u8, parse_result: &ParseResult, cache_data: &mut CacheData, tx_conf: &mut TxConf) {
        let pipeline_args = PipelineArgs {
            table_list: &self.table_list,
            pkt,
            parse_result,
            is_cache: false,
            cache_data,
            tx_conf,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_args as *const PipelineArgs as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }


    pub fn run_cache_pipeline(&mut self, pkt: *mut u8, parse_result: &ParseResult, cache_data: &mut CacheData, tx_conf: &mut TxConf) {
        let pipeline_cache_args = PipelineArgs {
            table_list: &self.table_list,
            pkt,
            parse_result,
            is_cache: true,
            cache_data,
            tx_conf,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_cache_args as *const PipelineArgs as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }
}
