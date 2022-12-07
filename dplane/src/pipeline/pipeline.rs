use std::sync::RwLock;

use crate::core::memory::array::Array;
// use crate::core::memory::ring::RingBuf;
use crate::core::runtime::wasm::runtime;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::table::Table;
use crate::pipeline::output::Output;

// runtime native api
use crate::pipeline::runtime_native_api::PipelineArgs;
use crate::pipeline::runtime_native_api::debug;
use crate::pipeline::runtime_native_api::table_search;
use crate::pipeline::runtime_native_api::pkt_get_header_len;
use crate::pipeline::runtime_native_api::pkt_get_payload_len;
use crate::pipeline::runtime_native_api::pkt_read;
use crate::pipeline::runtime_native_api::pkt_write;
use crate::pipeline::runtime_native_api::metadata_read;
use crate::pipeline::runtime_native_api::action_get_id;
use crate::pipeline::runtime_native_api::action_get_data;
use crate::pipeline::runtime_native_api::output_port;
use crate::pipeline::runtime_native_api::output_all;
use crate::pipeline::runtime_native_api::output_controller;
use crate::pipeline::runtime_native_api::output_drop;


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
                "s4_sys_table_search" => table_search,
                "s4_sys_pkt_get_header_len" => pkt_get_header_len,
                "s4_sys_pkt_get_payload_len" => pkt_get_payload_len,
                "s4_sys_pkt_read" => pkt_read,
                "s4_sys_pkt_write" => pkt_write,
                "s4_sys_metadata_read" => metadata_read,
                "s4_sys_action_get_id" => action_get_id,
                "s4_sys_action_get_data" => action_get_data,
                "s4_sys_output_port" => output_port,
                "s4_sys_output_all" => output_all,
                "s4_sys_output_controller" => output_controller,
                "s4_sys_output_drop" => output_drop,
            }
        );

        let runtime_args = runtime::new_runtime_args!(1);

        Pipeline {
            runtime,
            runtime_args,
            table_list,
        }
    }


    pub fn run_pipeline(&mut self, pkt: *mut u8, pkt_len: usize, parse_result: &ParseResult, cache_data: &mut CacheData, output: &mut Output) {
        let pipeline_args = PipelineArgs {
            table_list: &self.table_list,
            pkt,
            pkt_len,
            parse_result,
            is_cache: false,
            cache_data,
            output,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_args as *const PipelineArgs as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }


    pub fn run_cache_pipeline(&mut self, pkt: *mut u8, pkt_len: usize, parse_result: &ParseResult, cache_data: &mut CacheData, output: &mut Output) {
        let pipeline_cache_args = PipelineArgs {
            table_list: &self.table_list,
            pkt,
            pkt_len,
            parse_result,
            is_cache: true,
            cache_data,
            output,
        };
        runtime::set_runtime_arg_i64!(self.runtime_args, 0, &pipeline_cache_args as *const PipelineArgs as i64);
        runtime::call_runtime!(self.runtime, "run_pipeline", self.runtime_args);
    }
}
