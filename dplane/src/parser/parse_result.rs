use crate::core::memory::array::Array; 
use crate::parser::parser::Parser;

pub struct ParseResult {
    // pub parser: &'a Parser<'a>,
    pub hdr_len: usize,
    pub parse_result_of_header_list: Array<ParseResultOfHeader>,
}

pub struct ParseResultOfHeader {
    pub is_valid: bool,
    pub offset: u16,
}

// impl<'a> ParseResult<'a> {
//     pub fn free(&'a mut self) {
//         self.parser.parse_result_free(self);
//     }
// } 
