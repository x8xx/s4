#![no_main]
mod libparser;
use libparser::*;


#[no_mangle]
pub fn parse(parser_args_ptr: i64) -> bool {
    let pkt_len = unsafe { s4_sys_get_pkt_len(parser_args_ptr) };
    parse_ethernet(parser_args_ptr, pkt_len)
}

fn parse_ethernet(parser_args_ptr: i64, pkt_len: usize) -> bool {
    let hdr_id = 0;
    let ethernet_size = 14;
    if pkt_len <= ethernet_size {
        return false;
    }

    unsafe {
        s4_sys_extract_hdr(parser_args_ptr, hdr_id, 0, ethernet_size as i32);
    };
    true
}
