use crate::core::memory::array::Array; 

pub struct ParseResult {
    pub metadata: Metadata,
    pub hdr_size: usize,
    pub header_list: Array<Header>,
}

pub struct Metadata {
    pub port: u8,
    pub is_drop: bool,
    pub is_flooding: bool,
}

pub struct Header {
    pub is_valid: bool,
    pub offset: u16,
}
