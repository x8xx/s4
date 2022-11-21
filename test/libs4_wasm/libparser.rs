extern {
    pub fn s4_sys_get_pkt_len(parser_args_ptr: i64) -> usize;
    pub fn s4_sys_read_pkt(parser_args_ptr: i64, offset: u8) -> u8;
    pub fn s4_sys_extract_hdr(parser_args_ptr: i64, hdr_id: i64, offset: i32, hdr_size: i32);
}
