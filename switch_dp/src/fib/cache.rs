use crate::dpdk::dpdk_memory;

pub struct CacheElement<'a> {
    key: &'a mut [u8],
    key_len: u8,
    action_id: u8,
}


impl<'a> CacheElement<'a> {
    pub fn clean(&mut self, key: &'a mut [u8]) {
        self.key = key;
        self.key_len = 0;
        self.action_id = 0;
    }

    pub fn compare_key(&self, key: &[u8]) -> Option<u8> {
        if self.key_len == 0 {
            return None;
        }
        for i in 0..self.key_len {
            if self.key[i as usize] != key[i as usize] {
                return None;
            }
        }
        Some(self.action_id)
    }

    pub fn update(&mut self, key: &'a [u8], action_id: u8) {
        self.key_len = key.len() as u8;
        for i in 0..key.len() {
                self.key[i] = key[i];
        }
        self.action_id = action_id;
    }
}


// pub struct Key<'a> {
//     key: &'a  mut [u8],
//     len: usize,
// }


// impl<'a> Key<'a> {
//     pub fn new(key: &'a mut [u8]) -> Self {
//         Key {
//             key,
//             len: 0,
//         }
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cache_element() {
        // let key1: [u8; 3] = [0xff, 0x10, 0x0f];
        // let mut ce = CacheElement::new(&key1, 3);
        // assert_ne!(ce.compare_key(&key1), None);
        // assert_eq!(ce.compare_key(&key1), Some(3));
        // let key2: [u8; 3] = [0xff, 0x20, 0x0f];
        // assert_eq!(ce.compare_key(&key2), None);
        // ce.update(&key2, 4);
        // assert_ne!(ce.compare_key(&key2), None);
        // assert_eq!(ce.compare_key(&key2), Some(4));
    }
}
