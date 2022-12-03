use crate::core::memory::array::Array; 

pub struct ParseResult {
    pub metadata: Array<u32>,
    pub hdr_size: usize,
    pub header_list: Array<Header>,
}

pub enum Metadata {
    InPort = 0,
}


pub struct Header {
    pub is_valid: bool,
    pub offset: u16,
}
