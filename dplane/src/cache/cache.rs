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
                if *self.key.offset(i) != *ptr_key.offset(i) {
                    return false;
                }
            }
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use super::CacheElement;
    use crate::core::memory::array::Array;

    #[test]
    pub fn test_cmp_ptr_key() {
        let mut cache_element = CacheElement {
            key: Array::new(0).as_ptr(),
            key_len: 0,
            data: Array::new(3),
        };

        let mut key = Array::<u8>::new(10);
        key[0] = 10;
        key[1] = 20;
        key[2] = 30;
        key[3] = 40;
        key[4] = 50;
        cache_element.key = key.as_ptr();
        cache_element.key_len = 5;

        let mut target_key = Array::<u8>::new(10);
        assert!(!cache_element.cmp_ptr_key(target_key.as_ptr(), 5));

        target_key[0] = 10;
        target_key[1] = 20;
        target_key[2] = 30;
        target_key[3] = 40;
        target_key[4] = 50;

        assert!(!cache_element.cmp_ptr_key(target_key.as_ptr(), 4));
        assert!(cache_element.cmp_ptr_key(target_key.as_ptr(), 5));
    }
}
