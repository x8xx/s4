use std::ptr::null;
use crate::fib::cache::CacheElement;


pub struct L1Cache {
    cache: *mut CacheElement,
    cache_len: u32,
    key_len: u8,
}


impl L1Cache {
    pub fn new(cache: *mut CacheElement, cache_len: u32, key_len: u8) -> Self {
        L1Cache {
            cache,
            cache_len,
            key_len,
        }
    }

    pub fn update(&self, index: u32, key: *const u8, action_id: u8) -> bool {
        if index >= self.cache_len {
            return false;
        }
        unsafe {
            (*self.cache.offset(index as isize)).key = key;
            (*self.cache.offset(index as isize)).action_id = action_id;
        }
        true
    }

    pub fn delete(&self, index: u32) -> bool {
        self.update(index, null(), 0)
    }

    pub fn find(&self, index: u32, key: *const u8) -> bool {
        if index >= self.cache_len && 
            unsafe { !self.cache.offset(index as isize).is_null() } {
            return false;
        }

        unsafe {
            (*self.cache.offset(index as isize)).compare_key(key, self.key_len)
        }
    }
}
