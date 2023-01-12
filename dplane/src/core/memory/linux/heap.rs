use std::sync::RwLock;
use std::mem::size_of;
use std::ptr::null_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::core::memory::vector::VectorMeta;


pub static mut HEAP: RwLock<Heap> = RwLock::new(Heap {
    memzones: null_mut(), 
    current_memzone: 0,
    max_memzone_num: 10,
    // memzone_data_size: 536870912,
    memzone_data_size: 1073741824,
    size: 0,
    next_pos: 0,
});


#[derive(Clone, Copy)]
pub struct Heap {
    memzones: *mut *mut u8,
    current_memzone: isize,
    max_memzone_num: isize,
    memzone_data_size: isize,
    size: usize,
    next_pos: isize,
}


macro_rules! get_memzone_data {
    ($memzone: expr) => {
        $memzone
    }
}


impl Heap {
    pub fn new() -> &'static RwLock<Self> {
        unsafe {
            &HEAP
        }
    }


    pub fn init(memzone_data_size: usize, max_memzone_num: usize) {
        let memzones = unsafe {
            libc::malloc(max_memzone_num * size_of::<*mut u8>()) as *mut *mut u8
        };

        unsafe {
            for i in 0..max_memzone_num {
                let memzone = libc::malloc(memzone_data_size * size_of::<u8>()) as *mut u8;
                *memzones.offset(i as isize) = memzone;
            }
        }


        unsafe {
            let mut heap = HEAP.write().unwrap();
            heap.memzones = memzones;
            heap.max_memzone_num = max_memzone_num as isize;
            heap.memzone_data_size = memzone_data_size as isize;

//             HEAP = RwLock::new(Heap {
//                 memzones,
//                 current_memzone: 0,
//                 max_memzone_num: max_memzone_num as isize,
//                 memzone_data_size: memzone_data_size as isize,
//                 size: 0,
//                 next_pos: 0,
//             });
        }
        
    }


    pub fn malloc<T>(&mut self, size: usize) ->  *mut T {
        let mut start_pos = self.next_pos;
        let mut end_pos = start_pos + (size_of::<T>() * size) as isize;
        let memzone = unsafe {
            if end_pos > self.memzone_data_size {
                self.current_memzone += 1;
                start_pos = 0;
                end_pos = start_pos + (size_of::<T>() * size) as isize;

                if end_pos > self.memzone_data_size {
                    panic!("Failed alloc memory! over max memory size");
                }
            }
            *self.memzones.offset(self.current_memzone)
        };
        self.next_pos= end_pos as isize + 1;

        unsafe {
            get_memzone_data!(memzone).offset(start_pos) as *mut T
        }
    }
}


// pub struct Heap {
//     data: *mut u8,
//     size: usize,
//     next_pos: isize,
// }


// impl Heap {
//     pub fn new(size: usize) -> Self {
//         let data = if size != 0 {
//             unsafe {
//                 libc::malloc(size * size_of::<u8>()) as *mut u8 
//             }
//         } else {
//             null_mut() as *mut u8
//         };

//         Heap {
//             data,
//             size,
//             next_pos: 0,
//         }
//     }


//     pub fn malloc<T>(&mut self, size: usize) -> Array<T> {
//         let start_pos = self.next_pos;
//         let end_pos = start_pos + (size_of::<T>() * size) as isize;
//         self.next_pos= end_pos as isize + 1;

//         unsafe {
//             Array::new_manual(self.data.offset(start_pos) as *mut T, size)
//         }
//     }


//     pub fn vec_malloc<T: Copy>(&mut self, size: usize, extend_size: usize) -> Vector<T> {
//         // data
//         let data_pos = self.next_pos;
//         let end_pos = data_pos + (size_of::<T>() * size) as isize;
//         self.next_pos= end_pos as isize + 1;

//         // meta
//         let meta_pos = self.next_pos;
//         let end_pos = meta_pos + size_of::<VectorMeta>() as isize;
//         self.next_pos= end_pos as isize + 1;

//         unsafe {
//             let meta = self.data.offset(meta_pos) as *mut VectorMeta;
//             (*meta).pos = 0;
//             (*meta).len = size;
//             (*meta).extend_size = extend_size;
//             Vector::new_manual(self.data.offset(data_pos) as *mut T, meta)
//         }
//     }


//     pub fn free(&self) {
//         unsafe {
//             libc::free(self.data as *mut c_void)
//         }
//     }
// }
