use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheData;
use crate::pipeline::table::Table;
use crate::pipeline::table::FlowEntry;
use crate::pipeline::table::ActionSet;
use crate::pipeline::output::Output;


pub struct PipelineArgs<'a> {
    pub table_list: &'a Array<RwLock<Table>>,
    pub pkt: *mut u8,
    pub pkt_len: usize,
    pub parse_result: &'a ParseResult,
    pub is_cache: bool,
    pub cache_data: &'a mut CacheData,
    pub output: &'a mut Output,
}


pub fn debug(id: i64) {
    println!("api debug {}", id);
}


pub fn table_search(pipeline_args_ptr: i64, table_id: i32) -> i64 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    let PipelineArgs { table_list, pkt, pkt_len: _, parse_result, is_cache, cache_data, output: _ } = pipeline_args;

    if *is_cache {
        &unsafe { &*cache_data[table_id as usize] }.action as *const ActionSet as i64
    } else {
        let table = table_list[table_id as usize].read().unwrap();
        let flow_entry = table.search(*pkt as *const u8, *parse_result);
        cache_data[table_id as usize] = flow_entry as *const FlowEntry;
        &flow_entry.action as *const ActionSet as i64
    }
}


pub fn pkt_get_header_len(pipeline_args_ptr: i64) -> i32 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    pipeline_args.parse_result.hdr_size as i32
}


pub fn pkt_get_payload_len(pipeline_args_ptr: i64) -> i32 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    (pipeline_args.pkt_len - pipeline_args.parse_result.hdr_size) as i32
}


pub fn pkt_read(pipeline_args_ptr: i64, offset: i32) -> i32 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    unsafe {
        *(pipeline_args.pkt.offset(offset as isize)) as i32
    }
}


pub fn pkt_write(pipeline_args_ptr: i64, offset: i32, value: i32) {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    unsafe {
        *(pipeline_args.pkt.offset(offset as isize)) = value as u8;
    }
}


pub fn metadata_read(pipeline_args_ptr: i64, metadata_id: i32) -> i32 {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    pipeline_args.parse_result.metadata[metadata_id as usize] as i32

}


pub fn action_get_id(action_set_ptr: i64) -> i32 {
    let action_set = unsafe { & *(action_set_ptr as *const ActionSet) };
    action_set.action_id as i32
}


pub fn action_get_data(action_set_ptr: i64, index: i32) -> i32 {
    let action_set = unsafe { & *(action_set_ptr as *const ActionSet) };
    action_set.action_data[index as usize] as i32
}


pub fn output_port(pipeline_args_ptr: i64, port: i32) {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    *pipeline_args.output = Output::Port(port as u8);
}


pub fn output_all(pipeline_args_ptr: i64) {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    *pipeline_args.output = Output::All;
}


pub fn output_controller(pipeline_args_ptr: i64) {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    *pipeline_args.output = Output::Controller;
}


pub fn output_drop(pipeline_args_ptr: i64) {
    let pipeline_args = unsafe { &mut *(pipeline_args_ptr as *mut PipelineArgs) };
    *pipeline_args.output = Output::Drop;
}



//pub fn read_pkt(pipeline_args_ptr: i64, offset: i32) -> i32 {
//    let pkt_ptr = unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        pipeline_args.pkt
//    };
//    // let pkt_ptr = pkt_ptr as *const u8;
//    unsafe {
//        *(pkt_ptr.offset(offset as isize)) as i32
//    }
//}


//pub fn write_pkt(pipeline_args_ptr: i64, offset: u8, value: u8) {
//    let pkt_ptr = unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        pipeline_args.pkt
//    };
//    // let pkt_ptr = pkt_ptr as *mut u8;
//    //
//    unsafe {
//        *(pkt_ptr.offset(offset as isize)) = value;
//    }
//}


//pub fn get_metadata(pipeline_args_ptr: i64, metadata_id: i32) -> i32 {
//    let parse_result = unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        &pipeline_args.parse_result
//    };
//    // let parse_result = unsafe {
//    //     &*(parse_result_ptr as *const ParseResult)
//    // };
//    match metadata_id {
//        0 => (*parse_result).metadata.port as i32,
//        _ => 0,
//    }
//}


//pub fn set_metadata(pipeline_args_ptr: i64, metadata_id: i32, value: i64) {
//    let tx_conf= unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        &mut pipeline_args.tx_conf
//    };
//    match metadata_id {
//        0 => { (*tx_conf).output_port = value as usize },
//        _ => {},
//    }
//}


//pub fn get_action_id(action_set_ptr: i64) -> i32 {
//    let action_set = unsafe {
//        & *(action_set_ptr as *const ActionSet)
//    };
//    action_set.action_id as i32
//}


//pub fn get_action_data(action_set_ptr: i64, index: i32) -> i32 {
//    let action_set = unsafe {
//        & *(action_set_ptr as *const ActionSet)
//    };
//    action_set.action_data[index as usize]
//}


//pub fn to_controller(pipeline_args_ptr: i64) {
//    let tx_conf= unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        &mut pipeline_args.tx_conf
//    };

//    (*tx_conf).output_port = 0;
//}


//pub fn drop(pipeline_args_ptr: i64) {
//    let tx_conf= unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        &mut pipeline_args.tx_conf
//    };

//    (*tx_conf).is_drop = true;
//}

//pub fn flooding(pipeline_args_ptr: i64) {
//    let tx_conf= unsafe {
//        let pipeline_args = &mut *(pipeline_args_ptr as *mut PipelineArgs);
//        &mut pipeline_args.tx_conf
//    };

//    (*tx_conf).is_flooding= true;
//}
