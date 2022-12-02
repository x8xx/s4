use crate::parser::parse_result::ParseResult;


pub struct DeparserArgs {
    pub pkt: *mut u8,
    pub parse_result: *mut ParseResult,
}


pub fn emit(deparser_args_ptr: i64, header_id: i32, is_fix_hdr: i32) {
    let deparser_args = unsafe { &mut *(deparser_args_ptr as *mut DeparserArgs) };
    let DeparserArgs { pkt, parse_result } = deparser_args;

}
