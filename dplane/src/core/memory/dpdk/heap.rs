use std::sync::RwLock;
use std::mem::size_of;
use std::ptr::null_mut;
use std::os::raw::c_char;


pub static mut HEAP: RwLock<Heap> = RwLock::new(Heap {
    memzones: null_mut(), 
    current_memzone: 0,
    max_memzone_num: 10,
    // memzone_data_size: 536870912,
    memzone_data_size: 1073741824,
    next_pos: 0,
});


#[derive(Clone, Copy)]
pub struct Heap {
    memzones: *mut *const dpdk_sys::rte_memzone,
    current_memzone: isize,
    max_memzone_num: isize,
    memzone_data_size: isize,
    next_pos: isize,
}



macro_rules! get_memzone_data {
    ($memzone: expr) => {
        (*$memzone).__bindgen_anon_1.addr
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
            let memzone = dpdk_sys::rte_memzone_reserve(
                crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                size_of::<*const dpdk_sys::rte_memzone>() * max_memzone_num,
                dpdk_sys::rte_socket_id() as i32,
                dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
             );
            (*memzone).__bindgen_anon_1.addr as *mut *const dpdk_sys::rte_memzone
        };

        unsafe {
            for i in 0..max_memzone_num {
                *memzones.offset(i as isize) = dpdk_sys::rte_memzone_reserve(
                    crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                    size_of::<u8>() * memzone_data_size,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
                );
            }
        }


        unsafe {
            HEAP = RwLock::new(Heap {
                memzones,
                current_memzone: 0,
                max_memzone_num: max_memzone_num as isize,
                memzone_data_size: memzone_data_size as isize,
                next_pos: 0,
            });
        }
        
    }


    pub fn malloc<T>(&mut self, size: usize) ->  *mut T {
        // println!("start_pos: {}, end_pos:", self.next_pos);
        if size == 0 {
            return null_mut();
        }
        let mut start_pos = self.next_pos;
        let mut end_pos = start_pos + (size_of::<T>() * size) as isize;
        let memzone = unsafe {
            if end_pos > self.memzone_data_size {
                println!("usoyona? {}", self.memzone_data_size);
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
            // Array::new_manual(self.data.offset(start_pos) as *mut T, size)
        }
    }


    // pub fn vec_malloc<T: Copy>(&mut self, size: usize, extend_size: usize) -> Vector<T> {
    //     // data
    //     let data_pos = self.next_pos;
    //     let end_pos = data_pos + (size_of::<T>() * size) as isize;
    //     self.next_pos= end_pos as isize + 1;

    //     // meta
    //     let meta_pos = self.next_pos;
    //     let end_pos = meta_pos + size_of::<VectorMeta>() as isize;
    //     self.next_pos= end_pos as isize + 1;

    //     unsafe {
    //         let meta = self.data.offset(meta_pos) as *mut VectorMeta;
    //         (*meta).pos = 0;
    //         (*meta).len = size;
    //         (*meta).extend_size = extend_size;
    //         Vector::new_manual(self.data.offset(data_pos) as *mut T, meta)
    //     }
    // }


    // pub fn free(&self) {
    //     unsafe {
    //         dpdk_sys::rte_memzone_free(self.memzone);
    //     }
    // }
}
