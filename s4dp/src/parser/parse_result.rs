use crate::core::memory::array::Array; 

pub struct ParseResult {
    pub hdr_len: usize,
    pub parse_result_of_header_list: Array<ParseResultOfHeader>,
}

pub struct ParseResultOfHeader {
    pub is_valid: bool,
    pub offset: u16,
}
