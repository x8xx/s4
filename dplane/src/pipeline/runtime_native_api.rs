use std::sync::RwLock;
use crate::core::runtime::wasm::runtime;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::table::Table;
use crate::pipeline::table::ActionSet;


pub struct PipelineArgs<'a> {
    pub table_list: &'a Array<RwLock<Table>>,
    pub pkt: *mut u8,
    pub parse_result: &'a mut ParseResult,
    pub cache_runtime_args: &'a mut runtime::RuntimeArgs,
}


pub fn debug(id: i64) {
    println!("api debug {}", id);
}

pub fn search_table(pipeline_args_ptr: i64, table_id: i32) -> i64 {
    let (table, pkt, parse_result, cache_runtime_args) = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);

        let table_list = pipeline_args.table_list;
        let table = &table_list[table_id as usize];
        let pkt = pipeline_args.pkt;
        let parse_result = &pipeline_args.parse_result;
        let cache_runtime_args = &mut pipeline_args.cache_runtime_args;

        // let table_list = &mut *(table_list_ptr as *mut Array<RwLock<Table>>);
        // let table = &mut table_list[table_id as usize];
        // let pkt = pkt_ptr as *const u8;
        // let parse_result = & *(parse_result_ptr as *const ParseResult);
        (table, pkt, parse_result, cache_runtime_args)
    };

    let table = table.read().unwrap();
    let action_set = table.search(pkt, *parse_result);
    
    runtime::set_runtime_arg_i64!(*cache_runtime_args, table_id as usize, action_set as *const ActionSet as i64);

    action_set as *const ActionSet as i64
}


pub fn read_pkt(pipeline_args_ptr: i64, offset: i32) -> i32 {
    let pkt_ptr = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        pipeline_args.pkt
    };
    // let pkt_ptr = pkt_ptr as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}


pub fn write_pkt(pipeline_args_ptr: i64, offset: u8, value: u8) {
    let pkt_ptr = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        pipeline_args.pkt
    };
    // let pkt_ptr = pkt_ptr as *mut u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) = value;
    }
}


pub fn get_metadata(pipeline_args_ptr: i64, metadata_id: i32) -> i32 {
    let parse_result = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &pipeline_args.parse_result
    };
    // let parse_result = unsafe {
    //     &*(parse_result_ptr as *const ParseResult)
    // };
    match metadata_id {
        0 => (*parse_result).metadata.port as i32,
        _ => 0,
    }
}


pub fn set_metadata(pipeline_args_ptr: i64, metadata_id: i32, value: i64) {
    let parse_result = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.parse_result
    };
    match metadata_id {
        0 => { (*parse_result).metadata.port = value as u8 },
        _ => {},
    }
}


pub fn get_action_id(action_set_ptr: i64) -> i32 {
    let action_set = unsafe {
        & *(action_set_ptr as *const ActionSet)
    };
    action_set.action_id as i32
}


pub fn get_action_data(action_set_ptr: i64, index: i32, offset: i32) -> i32 {
    let action_set = unsafe {
        & *(action_set_ptr as *const ActionSet)
    };
    let action_data = &action_set.action_data[index as usize];
    action_data[offset as usize]
}


pub fn to_controller(pipeline_args_ptr: i64) {
    // TODO: push to_controller_ring
    let parse_result = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.parse_result
    };
    // let parse_result = unsafe {
    //     &mut *(parse_result_ptr as *mut ParseResult)
    // };
    (*parse_result).metadata.is_drop = true;
}


pub fn drop(pipeline_args_ptr: i64) {
    let parse_result = unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.parse_result
    };
    // let parse_result = unsafe {
    //     &mut *(parse_result_ptr as *mut ParseResult)
    // };
    (*parse_result).metadata.is_drop = true;
}
