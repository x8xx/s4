use crate::fib::header::*;

pub struct WasmParserArgs {
    pub hdrs: *mut Header,
    pub parsed_hdr: *mut (u8, *mut Header),
    pub size: isize,
}

pub fn wasm_pkt_read(packet_id: i64, offset: i32) -> i32 {
    unsafe {
        (*(packet_id as *const u8).offset(offset as isize)) as i32
    }
}

pub fn wasm_extract_header(parse_id: i64, hdr_id: i64, base_offset: i64) {
    unsafe {
        let wasm_parser_args= &mut *(parse_id as *mut WasmParserArgs);
        let new_hdr = wasm_parser_args.parsed_hdr.offset(wasm_parser_args.size);
        (*new_hdr).0 = base_offset as u8;
        (*new_hdr).1 = wasm_parser_args.hdrs.offset(hdr_id as isize);
        wasm_parser_args.size += 1;
    }
}
