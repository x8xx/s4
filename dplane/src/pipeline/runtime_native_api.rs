use std::sync::RwLock;
use crate::core::logger::log::*;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::table::Table;
use crate::pipeline::table::FlowEntry;
use crate::pipeline::table::ActionSet;
use crate::pipeline::output::Output;


pub struct RuntimeArgs<'a, 'b> {
    pub table_list: &'a Array<RwLock<Table>>,
    pub pkt: *mut u8,
    pub pkt_len: usize,
    pub parse_result: &'b ParseResult,
    pub is_cache: bool,
    pub cache_data: &'b mut CacheData,
    pub output: &'b mut Output,
}


pub fn debug(id: i64) {
    println!("api debug {}", id);
}


/**
 * table
 */

pub fn table_search(runtime_args_ptr: i64, table_id: i32) -> i64 {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    let RuntimeArgs { table_list, pkt, pkt_len: _, parse_result, is_cache, cache_data, output: _ } = runtime_args;

    if *is_cache {
        &unsafe { &*cache_data[table_id as usize] }.action as *const ActionSet as i64
    } else {
        let table = table_list[table_id as usize].read().unwrap();
        debug_log!("table search start");
        let flow_entry = table.search(*pkt as *const u8, *parse_result);
        debug_log!("table search done");
        cache_data[table_id as usize] = flow_entry as *const FlowEntry;
        &flow_entry.action as *const ActionSet as i64
    }
}


/**
 * pkt 
 */

pub fn pkt_get_header_len(runtime_args_ptr: i64) -> i32 {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    runtime_args.parse_result.hdr_size as i32
}


pub fn pkt_get_payload_len(runtime_args_ptr: i64) -> i32 {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    (runtime_args.pkt_len - runtime_args.parse_result.hdr_size) as i32
}


pub fn pkt_read(runtime_args_ptr: i64, offset: i32) -> i32 {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    unsafe {
        *(runtime_args.pkt.offset(offset as isize)) as i32
    }
}


pub fn pkt_write(runtime_args_ptr: i64, offset: i32, value: i32) {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    unsafe {
        *(runtime_args.pkt.offset(offset as isize)) = value as u8;
    }
}

pub fn pkt_alloc_payload(runtime_args_ptr: i64, start_offset: i32, size: i32) {

}


/**
 * metadata
 */

pub fn metadata_read(runtime_args_ptr: i64, metadata_id: i32) -> i32 {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    runtime_args.parse_result.metadata[metadata_id as usize] as i32

}


/**
 * action
 */

pub fn action_get_id(action_set_ptr: i64) -> i32 {
    let action_set = unsafe { & *(action_set_ptr as *const ActionSet) };
    action_set.action_id as i32
}


pub fn action_get_data(action_set_ptr: i64, index: i32) -> i32 {
    let action_set = unsafe { & *(action_set_ptr as *const ActionSet) };
    action_set.action_data[index as usize] as i32
}


/**
 * output
 */

pub fn output_port(runtime_args_ptr: i64, port: i32) {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    *runtime_args.output = Output::Port(port as u8);
}


pub fn output_all(runtime_args_ptr: i64) {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    *runtime_args.output = Output::All;
}


pub fn output_controller(runtime_args_ptr: i64) {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    *runtime_args.output = Output::Controller;
}


pub fn output_drop(runtime_args_ptr: i64) {
    let runtime_args = unsafe { &mut *(runtime_args_ptr as *mut RuntimeArgs) };
    *runtime_args.output = Output::Drop;
}
