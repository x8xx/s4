use crate::dpdk::dpdk_memory::malloc;

pub struct Cache {
    data: *mut u8,
    pub max_table_len: u8,
    pub max_key_len: u8,
}

impl Cache {
    pub fn new() -> Self {

    }

    pub fn get(&self) -> *mut u8 {
        self.data
    }

    pub fn set(&self) {

    }
}

pub fn key_compare_slice_pointer(slice_key: &[u8], pointer_key: *const u8) -> bool {
    for (i, byte) in slice_key.iter().enumerate() {
        unsafe {
            if *byte != *pointer_key.offset(i as isize) {
                return false;
            }
        }
    }
    true
}
