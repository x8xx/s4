use crate::parser::parse_result::ParseResult;

pub fn read_pkt(pkt_ptr: i64, offset: i32) -> i32 {
    let pkt_ptr = pkt_ptr as *const u8;
    unsafe {
        *(pkt_ptr.offset(offset as isize)) as i32
    }
}

pub fn extract_hdr(parse_result_ptr: i64, hdr_id: i64, offset: i32, hdr_size: i32) {
    let parse_result = unsafe { &mut *(parse_result_ptr as  *mut ParseResult) as &mut ParseResult };
    parse_result.hdr_len += hdr_size as usize;
    parse_result.header_list[hdr_id as usize].is_valid = true;
    parse_result.header_list[hdr_id as usize].offset = offset.try_into().unwrap();
}
