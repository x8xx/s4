use crate::fib::cache::CacheElement;


pub struct LbFilter {
    filter: *mut u8,
    filter_len: u32,
}


impl LbFilter {
    pub fn new(filter: *mut u8, filter_len: u32) -> Self {
        LbFilter {
            filter,
            filter_len,
        }
    }
}
