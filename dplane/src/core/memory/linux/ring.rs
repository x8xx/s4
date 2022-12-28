// TODO

use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;
use crate::core::memory::array::Array;


#[derive(Clone)]
pub struct Ring {
    ring: *mut c_void,
    current_enqueue_pos: *mut isize,
    current_dequeue_pos: *mut isize,
}

unsafe impl<'a> Send for Ring {}

impl Ring {
    pub fn new(len: usize) -> Self {
        let (ring, current_enqueue_pos, current_dequeue_pos) = {
            unsafe {
                (
                    libc::malloc(len * size_of::<*mut c_void>()),
                    libc::malloc(size_of::<*mut isize>()) as *mut isize,
                    libc::malloc(size_of::<*mut isize>()) as *mut isize,
                )
            }
        };

        Ring {
            ring,
            current_enqueue_pos,
            current_dequeue_pos
        }
    }

    pub fn enqueue_burst<T>(&self, objs: &&mut T, len: usize) -> usize {
        0
        // unsafe {
        //     dpdk_sys::rte_ring_enqueue_burst(
        //         self.ring,
        //         objs as *const &mut T as *const *mut c_void,
        //         len as u32,
        //         null_mut()
        //     )
        // }.try_into().unwrap()
    }

    // pub fn dequeue_burst<T>(&self, objs: &mut &mut T, len: usize) -> usize {
    pub fn dequeue_burst<T>(&self, objs: &Array<&mut T>, len: usize) -> usize {
        0
        // unsafe {
        //     dpdk_sys::rte_ring_dequeue_burst(
        //         self.ring,
        //         objs.as_ptr() as *mut *mut T as *mut *mut c_void,
        //         len as u32,
        //         null_mut()
        //     )
        // }.try_into().unwrap()
    }

    pub fn dequeue_burst_resume<T>(&self, objs: &Array<&mut T>, pos: usize, len: usize) -> usize {
        0
    }

    pub fn enqueue<T>(&self, obj: &mut T) -> usize {
        0
        // unsafe {
        //     dpdk_sys::rte_ring_enqueue(
        //         self.ring,
        //         obj as *mut T as *mut c_void,
        //     )
        // }.try_into().unwrap()
    }

    pub fn dequeue<T>(&self, obj: &mut &mut T) -> usize {
        0
        // unsafe {
        //     dpdk_sys::rte_ring_dequeue(
        //         self.ring,
        //         obj as *mut &mut T as *mut *mut T as *mut *mut c_void,
        //     )
        // }.try_into().unwrap()
    }
}


pub struct RingBuf<T> {
    buf: *mut T,
    len: usize,
    current_enqueue_pos: *mut isize,
    current_dequeue_pos: *mut isize,
}

impl<T> RingBuf<T> {
    pub fn new(len: usize) -> Self {
        let (buf, current_enqueue_pos, current_dequeue_pos) = {
            unsafe {
                (
                    libc::malloc(len * size_of::<T>()) as *mut T,
                    libc::malloc(size_of::<*mut isize>()) as *mut isize,
                    libc::malloc(size_of::<*mut isize>()) as *mut isize,
                )
            }
        };


        RingBuf {
            buf,
            len,
            current_enqueue_pos,
            current_dequeue_pos
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn malloc_bulk(&self, obj: &mut [&mut T], len: usize) {
        // unsafe {
        //     let obj_ptr = obj as *mut [&mut T] as *mut *const T;
        //     dpdk_sys::rte_mempool_get_bulk(self.mempool, obj_ptr as *mut *mut c_void, len as u32);
        // }
    }

    pub fn free_bulk(&self, obj: &[&mut T], len: usize) {
        // unsafe {
        //     let obj_ptr = obj as *const [&mut T] as *const *mut T;
        //     dpdk_sys::rte_mempool_put_bulk(self.mempool, obj_ptr as *const *mut c_void, len as u32);
        // }
    }

    pub fn malloc<'a>(&'a self) -> &'a mut  T {
        unsafe {
            &mut *self.buf
        }
        // unsafe {
        //     let mut obj_ptr: *mut T = null_mut();
        //     dpdk_sys::rte_mempool_get(self.mempool, &mut obj_ptr as *mut *mut T as *mut *mut c_void);
        //     &mut *obj_ptr as &mut T
        // }
    }

    pub fn free(&self, obj: &mut T) {
        // unsafe {
        //     dpdk_sys::rte_mempool_put(self.mempool, obj as *mut T as *mut c_void);
        // }
    }
}

macro_rules! malloc_ringbuf_all_element {
    ($ringbuf: expr, $T: ident) => {
        {
            let ptr_array = Array::<&mut $T>:: new($ringbuf.len()); 
            $ringbuf.malloc_bulk(ptr_array.as_slice(), ptr_array.len());
            ptr_array
        }
    }

}

macro_rules! free_ringbuf_all_element {
    ($ringbuf: expr, $ptr_array: expr) => {
        $ringbuf.free_bulk($ptr_array.as_slice(), $ptr_array.len());
        $ptr_array.free();
    }

}

macro_rules! init_ringbuf_element {
    ($ringbuf: expr, $T: ident, { $( $field: ident => $value: expr, )* }) =>  {
        {
            let ptr_array = Array::<&mut $T>:: new($ringbuf.len()); 
            $ringbuf.malloc_bulk(ptr_array.as_slice(), ptr_array.len());

            for (_, element) in ptr_array.as_slice().iter_mut().enumerate() {
                $(
                    element.$field = $value;
                )*
            }

            $ringbuf.free_bulk(ptr_array.as_slice(), ptr_array.len());
            ptr_array.free();
        }
    }
}

pub(crate) use init_ringbuf_element;
pub(crate) use malloc_ringbuf_all_element;
pub(crate) use free_ringbuf_all_element;
