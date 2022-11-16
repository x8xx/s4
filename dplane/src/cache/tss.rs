use crate::core::memory::array::Array;
use crate::parser::header::Field;
use crate::cache::cache::CacheElement;

pub struct TupleSpace {
    tuple_list: Array<(Tuple, Array<CacheElement>)>,
    tuple_len: usize,
}

pub struct Tuple {
    fields: Array<Field>,
    values: Value,
}

struct Value {
    len: usize,
    value: Array<u8>,
}


impl TupleSpace {
    pub fn new(len: usize) -> Self {
        TupleSpace {
            tuple_list: Array::new(len),
            tuple_len: 0,
        }
    }

    pub fn search(&self) -> bool {
        true
    }
}


impl Tuple {
    pub fn new() {

    }
}


impl Value {
    pub fn new() {

    }
}
