pub struct CacheElement {
    pub key: *const u8,
    pub action_id: u8,
}


impl CacheElement {
    pub fn new(key: *const u8, action_id: u8) -> Self {
        CacheElement {
            key,
            action_id,
        }
    }

    pub fn compare_key(&self, key: *const u8, key_len: u8) -> bool {
        unsafe {
            if *(self.key) != *key {
                return false
            }

            for i in 1..key_len {
                if *(self.key.offset(i as isize)) != *key.offset(i as isize) {
                    return false
                }
            }
        }
        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cache_element_compare_key() {
        let mut ce_u8_key: u8 = 0xff;
        let mut ce_u8_key_ptr: *mut u8 = &mut ce_u8_key;
        let ce = CacheElement {
            key: ce_u8_key_ptr,
            action_id: 1,
        };
        let mut test1_key: u8 = 0xff;
        let mut test1_key_ptr: *mut u8 = &mut test1_key;
        assert!(ce.compare_key(test1_key_ptr, 1));
        test1_key = 0x64;
        assert!(!ce.compare_key(test1_key_ptr, 1));


        let mut ce_u16_key: [u8;2] = [0xff, 0x64];
        let mut ce_u16_key_ptr: *mut u8 =  ce_u16_key.as_ptr() as *mut u8;
        let ce = CacheElement {
            key: ce_u16_key_ptr,
            action_id: 1,
        };
        let mut test2_key: [u8;2] = [0xff, 0x64];
        let mut test2_key_ptr: *mut u8 =  test2_key.as_ptr() as *mut u8;
        assert!(ce.compare_key(test2_key_ptr, 2));
        test2_key[1] = 0xff;
        assert!(!ce.compare_key(test2_key_ptr, 2));
    }
}
