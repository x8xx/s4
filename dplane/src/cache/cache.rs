use crate::core::memory::array::Array;
use crate::pipeline::table::ActionSet;


pub type CacheData = Array<ActionSet>;

pub struct CacheElement {
    pub key: *const u8,
    pub key_len: isize,
    pub data: CacheData,
}

impl CacheElement {
    pub fn cmp_ptr_key(&self, ptr_key: *const u8, key_len: isize) -> bool {
        if key_len != self.key_len {
            return false;
        }

        for i in 0..key_len {
            unsafe {
                if self.key.offset(i) != ptr_key.offset(i) {
                    return false;
                }
            }
        }

        true
    }
}

pub struct CacheRelation<'a> {
    pub l1_cache_element: &'a CacheElement,
}
