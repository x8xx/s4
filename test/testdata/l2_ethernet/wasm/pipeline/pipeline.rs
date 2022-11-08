#![no_main]
mod libpipeline;
use libpipeline::*;


#[no_mangle]
pub fn run_pipeline(table_list_ptr: i64, pkt_ptr: i64, parse_result_ptr: i64) {
    let action_set_ptr = unsafe { search_table(table_list_ptr, 0, pkt_ptr, parse_result_ptr) };
    select_action(action_set_ptr, pkt_ptr, parse_result_ptr);
}

fn select_action(action_set_ptr: i64, pkt_ptr: i64, parse_result_ptr: i64) {
    let action_id = unsafe { get_action_id(action_set_ptr) };
    match action_id {
        0 => { action_set_port(action_set_ptr, pkt_ptr, parse_result_ptr) },
        1 => { action_drop(action_set_ptr, pkt_ptr, parse_result_ptr) },
        _ => { action_drop(action_set_ptr, pkt_ptr, parse_result_ptr) },
    }
}

fn action_set_port(action_set_ptr: i64, pkt_ptr: i64, parse_result_ptr: i64) {
    unsafe {
        let port = get_action_data(action_set_ptr, 0, 0);
        set_metadata(parse_result_ptr, 0, port as i64);
    }
}

fn action_drop(action_set_ptr: i64, pkt_ptr: i64, parse_result_ptr: i64) {
    unsafe {
        drop(parse_result_ptr);
    }
}
