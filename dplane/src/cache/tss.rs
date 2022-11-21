use crate::core::memory::array::Array;
use crate::parser::header::Field;
use crate::cache::cache::CacheData;
use crate::cache::cache::CacheElement;
use crate::cache::hash::l3_hash_function_murmurhash3;


pub struct TupleSpace<'a> {
    tuple_list: Array<(Tuple, Array<CacheElement>)>,
    tuple_len: usize,
    tuple_hash_table: Array<&'a Tuple>,
}

pub struct Tuple {
    fields: Array<Field>,
    values: Array<Value>,
    hash: u16,
    seed: u32,
}

struct Value {
    pub value: Array<u8>,
    pub prefix_mask: u8,
}

pub struct KeyStore {
    pub key: Array<u8>,
    pub key_len: usize,
}


impl<'a> TupleSpace<'a> {
    pub fn new(len: usize) -> Self {
        TupleSpace {
            tuple_list: Array::new(len),
            tuple_len: 0,
            tuple_hash_table: Array::new(65535),
        }
    }

    pub fn search(&self, pkt: *const u8, key_store: &mut KeyStore) -> Option<CacheData> {
        for i in 0..self.tuple_len {
            let hash = self.tuple_list[i].0.hash_function(pkt, key_store);
            match hash {
                Some(hash) => {
                    if self.tuple_list[i].1[hash as usize].cmp_ptr_key(key_store.key.as_ptr(), key_store.key_len as isize) {
                        return Some(self.tuple_list[i].1[hash as usize].data.clone());
                    }

                },
                None => {},
            }
        }
        None
    }

    // pub fn insert(&self, key: *const u8, key_len: isize) -> &CacheElement {

    // }
}


impl Tuple {
    // pub fn new() -> Self {

    // }

    pub fn hash_function(&self, pkt: *const u8, key_store: &mut KeyStore) -> Option<u16> {
        let mut key_next_offset = 0;
        for i in 0..self.fields.len() {
            key_next_offset += unsafe {
                if self.fields[i].cmp_pkt(pkt, 0, &self.values[i].value, self.values[i].prefix_mask) {
                    self.fields[i].copy_ptr_value(0, pkt as *mut u8, key_store.key.as_ptr().offset(key_next_offset))
                } else {
                    return None;
                }
            };
        }
        key_store.key_len = key_next_offset as usize;

        Some(l3_hash_function_murmurhash3(key_store.key.as_ptr(), key_store.key_len, self.seed))
    }
}


// impl Value {
//     pub fn new() {

//     }
// }


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
