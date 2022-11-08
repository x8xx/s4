use crate::core::memory::array::Array; 

pub struct ParseResult {
    pub metadata: Metadata,
    pub hdr_len: usize,
    pub header_list: Array<Header>,
}

pub struct Metadata {
    pub port: u8,
    pub is_drop: bool,
}

pub struct Header {
    pub is_valid: bool,
    pub offset: u16,
}
