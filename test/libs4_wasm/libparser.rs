extern {
    pub fn read_pkt(pkt_ptr: i64, offset: u8) -> u8;
    pub fn extract_hdr(parse_result_ptr: i64, hdr_id: i64, offset: i32, hdr_size: i32);
}

pub struct Packet {
    pub id: i64,
    pub len: u8,
    pub parse_result_id: i64,
}
