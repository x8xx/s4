use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::table::Table;
use crate::pipeline::table::ActionSet;
use crate::pipeline::tx_conf::TxConf;


pub struct PipelineArgs<'a> {
    pub table_list: &'a Array<RwLock<Table>>,
    pub pkt: *mut u8,
    pub parse_result: &'a ParseResult,
    pub new_cache_data: &'a mut CacheData,
    pub tx_conf: &'a mut TxConf,
}

pub struct PipelineCacheArgs<'a> {
    pub table_list: &'a Array<RwLock<Table>>,
    pub pkt: *mut u8,
    pub parse_result: &'a ParseResult,
    pub cache_data: &'a CacheData,
    pub tx_conf: &'a mut TxConf,
}


pub fn debug(id: i64) {
    println!("api debug {}", id);
}

pub fn search_table(pipeline_args_ptr: i64, table_id: i32) -> i64 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    let PipelineArgs { table_list, pkt, parse_result, new_cache_data, tx_conf: _ } = pipeline_args;

    let table = table_list[table_id as usize].read().unwrap();
    let action_set = table.search(*pkt as *const u8, *parse_result);
    new_cache_data[table_id as usize] = action_set.clone();
    action_set as *const ActionSet as i64
}


pub fn get_action_set_from_cache(pipeline_args_ptr: i64, table_id: i32) -> i64 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineCacheArgs) };
    let PipelineCacheArgs { table_list: _, pkt: _, parse_result: _, cache_data, tx_conf: _ } = pipeline_args;

    &cache_data[table_id as usize] as *const ActionSet as i64
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
    //
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
    let tx_conf= unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.tx_conf
    };
    match metadata_id {
        0 => { (*tx_conf).output_port = value as usize },
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
    let tx_conf= unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.tx_conf
    };

    (*tx_conf).output_port = 0;
}


pub fn drop(pipeline_args_ptr: i64) {
    let tx_conf= unsafe {
        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
        &mut pipeline_args.tx_conf
    };

    (*tx_conf).is_drop = true;
}
