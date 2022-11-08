use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::table::ActionSet;
use crate::pipeline::table::Table;


pub fn debug(id: i64) {
    println!("api debug {}", id);
}

pub fn search_table(table_list_ptr: i64, table_id: i32, pkt_ptr: i64, parse_result_ptr: i64) -> i64 {
    let (table, pkt, parse_result) = unsafe {
        let table_list = &mut *(table_list_ptr as *mut Array<Table>);
        let table = &mut table_list[table_id as usize];
        let pkt = pkt_ptr as *const u8;
        let parse_result = & *(parse_result_ptr as *const ParseResult);
        (table, pkt, parse_result)
    };

    table.search(pkt, parse_result) as *const ActionSet as i64
}


pub fn read_pkt(pkt_ptr: i64, offset: i32) -> i32 {
    let pkt_ptr = pkt_ptr as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}


pub fn write_pkt(pkt_ptr: i64, offset: u8, value: u8) {
    let pkt_ptr = pkt_ptr as *mut u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) = value;
    }
}


pub fn get_metadata(parse_result_ptr: i64, metadata_id: i32) -> i32 {
    let parse_result = unsafe {
        &*(parse_result_ptr as *const ParseResult)
    };
    match metadata_id {
        0 => parse_result.metadata.port as i32,
        _ => 0,
    }
}


pub fn set_metadata(parse_result_ptr: i64, metadata_id: i32, value: i64) {
    let parse_result = unsafe {
        &mut *(parse_result_ptr as *mut ParseResult)
    };
    match metadata_id {
        0 => { parse_result.metadata.port = value as u8 },
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


pub fn to_controller(pkt_ptr: i64, parse_result_ptr: i64) {
    // TODO: push to_controller_ring
    let parse_result = unsafe {
        &mut *(parse_result_ptr as *mut ParseResult)
    };
    parse_result.metadata.is_drop = true;
}


pub fn drop(parse_result_ptr: i64) {
    let parse_result = unsafe {
        &mut *(parse_result_ptr as *mut ParseResult)
    };
    parse_result.metadata.is_drop = true;
}
