use std::ptr::null_mut;
use crate::core::memory::array::Array;


pub struct LinearList<T> {
    pub data: Array<T>,
    pub pos: usize,
    pub next: *mut LinearList<T>,
}

impl <T> LinearList<T> {
    pub fn new(len: usize) -> Self {
        LinearList {
            data: Array::new(len),
            pos: 0,
            next: null_mut(),
        }
    }

    pub fn insert(&mut self, element: T) -> &mut T {
        let mut list = self;
        while list.pos == list.data.len() {
            if list.next == null_mut() {
                let mut mem = Array::<LinearList<T>>::new(1);
                mem.init(0, LinearList {
                    data: Array::new(list.data.len()),
                    pos: 0,
                    next: null_mut(),
                });
                list.next = mem.as_ptr();
                list = unsafe { &mut *list.next };
            }
            list = unsafe { &mut *list.next };
        }

        list.data.init(list.pos, element);
        list.pos += 1;

        list.data.get(list.pos - 1)
    }
}
