use crate::fib::header::ParsedHeader;

pub fn wasm_pkt_read(packet_id: i64, offset: i32) -> i32 {
    unsafe {
        (*(packet_id as *const u8).offset(offset as isize)) as i32
    }
}

pub fn wasm_extract_header(parse_id: i64, hdr_id: i64) {
    unsafe {
        // let parsed_header_ptr = parse_id as *mut ParsedHeader;
        // (*(*parsed_header_ptr).hdrs.offset((*parsed_header_ptr).pos)) = hdr_id as u8;
    }
}
