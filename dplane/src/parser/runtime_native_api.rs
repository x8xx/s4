use crate::parser::parse_result::ParseResult;


pub struct ParserArgs<'a> {
    pub pkt: *mut u8,
    pub pkt_len: usize,
    pub parse_result: *mut ParseResult,
    pub is_accept: &'a mut bool,
}


pub fn pkt_get_len(parser_args_ptr: i64) -> i32 {
    unsafe {
        (*(parser_args_ptr as *const ParserArgs)).pkt_len as i32
    }
}


pub fn pkt_read(parser_args_ptr: i64, offset: i32) -> i32 {
    unsafe {
        *(*(parser_args_ptr as *const ParserArgs)).pkt.offset(offset as isize) as i32
    }
}


pub fn pkt_drop(parser_args_ptr: i64) {
    let parser_args = unsafe { &mut *(parser_args_ptr as *mut ParserArgs) };
    *parser_args.is_accept = false;
}
 

pub fn extract_hdr(parser_args_ptr: i64, hdr_id: i64, offset: i32, hdr_size: i32) {
    let parse_result = unsafe { &mut *(*(parser_args_ptr as *const ParserArgs)).parse_result };

    parse_result.hdr_size += hdr_size as usize;
    parse_result.header_list[hdr_id as usize].is_valid = true;
    parse_result.header_list[hdr_id as usize].offset = offset.try_into().unwrap();
}
