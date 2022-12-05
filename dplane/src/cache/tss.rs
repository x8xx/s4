use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::parser::header::Field;
use crate::cache::cache::CacheData;
use crate::cache::cache::CacheElement;
use crate::cache::hash::l3_hash_function_murmurhash3;

pub struct L3Cache<'a> {
    pub l3_cache: *mut TupleSpace<'a>,
}
unsafe impl<'a> Send for L3Cache<'a> {}
unsafe impl<'a> Sync for L3Cache<'a> {}


pub type TupleField = (MatchKind, Field);

pub enum MatchKind {
    Lpm,
    Exact(Array<u8>, Array<u8>),
}

pub struct TupleSpace<'a> {
    tuple_list: Array<Tuple>,
    tuple_len: usize,
    tuple_hash_table: Array<&'a Tuple>,
}

pub struct Tuple {
    fields: Array<TupleField>,
    cache: Array<RwLock<CacheElement>>,
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
            let hash = self.tuple_list[i].hash_function(pkt, key_store);
            match hash {
                Some(hash) => {
                    let cache = self.tuple_list[i].cache[hash as usize].read().unwrap();
                    if cache.cmp_ptr_key(key_store.key.as_ptr(), key_store.key_len as isize) {
                        return Some(cache.data.clone());
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
    pub fn new(fields: Array<TupleField>, cache_len: usize, seed: u32) -> Self {
        Tuple {
            fields,
            cache: Array::new(cache_len),
            seed,
        }
    }


    pub fn hash_function(&self, pkt: *const u8, key_store: &mut KeyStore) -> Option<u16> {
        let mut key_next_offset = 0;
        for i in 0..self.fields.len() {
            let (kind, field) = &self.fields[i];
            match kind {
                MatchKind::Lpm => {
                    key_next_offset += unsafe {
                        field.copy_ptr_value(0, pkt as *mut u8, key_store.key.as_ptr().offset(key_next_offset))
                    };
                },
                MatchKind::Exact(start, end) => {
                    key_next_offset += unsafe {
                        if field.cmp_pkt_ge_value(pkt, 0, &start, 0xff) && field.cmp_pkt_le_value(pkt, 0, &end, 0xff) {
                            field.copy_ptr_value(0, pkt as *mut u8, key_store.key.as_ptr().offset(key_next_offset))
                        } else {
                            return None;
                        }
                    };
                }
            };
        }
        key_store.key_len = key_next_offset as usize;

        Some(l3_hash_function_murmurhash3(key_store.key.as_ptr(), key_store.key_len, self.seed))
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2 + 2, 4);
    }
}
