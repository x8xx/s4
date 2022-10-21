use crate::core::memory::array; 

pub struct ParseResult {
    pub hdr_len: usize,
    pub used_fields_index: usize,
}
