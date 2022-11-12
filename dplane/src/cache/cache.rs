use crate::core::runtime::wasm::runtime::RuntimeArgs;

pub struct CacheElement {
    key: *const u8,
    runtime_args: RuntimeArgs,
}

impl CacheElement {
    pub fn cmp_ptr_key(&self, ptr_key: *const u8) -> bool {
        // for (i, byte) in slice_key.iter().enumerate() {
        //     unsafe {
        //         if *byte != *self.key.offset(i as isize) {
        //             return false;
        //         }
        //     }
        // }
        true
    }

    pub fn cmp_slice_key(&self, slice_key: &[u8]) -> bool {
        for (i, byte) in slice_key.iter().enumerate() {
            unsafe {
                if *byte != *self.key.offset(i as isize) {
                    return false;
                }
            }
        }
        true
    }
}


//
// parse_result -> parse_hdr_flag
//
//
// cache_element
//  array<runtime_args> (table_record ptr
//  key
//
//
// l1 cache (array)
//  parser hdr_len -> pkt -> murmur
// lbf (array)
// l2 cahce (array)
//   see parse_hdr_flag -> parse_hdr_list[i] -> get offset and used fields -> pkt -> murmur
// l3 cache tss
//   
//  
