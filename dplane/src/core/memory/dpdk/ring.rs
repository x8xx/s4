use std::ptr::null_mut;
use std::mem::size_of;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;
use crate::core::memory::array::Array;


#[derive(Clone)]
pub struct Ring {
    ring: *mut dpdk_sys::rte_ring,
}

unsafe impl<'a> Send for Ring {}

impl Ring {
    pub fn new(len: usize) -> Self {
        let ring = {
            unsafe {
                dpdk_sys::rte_ring_create(
                    crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                    len as u32,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RING_F_MP_RTS_ENQ | dpdk_sys::RING_F_MC_RTS_DEQ
                    // dpdk_sys::RING_F_MP_HTS_ENQ | dpdk_sys::RING_F_MC_HTS_DEQ
                )
            }
        };

        Ring {
            ring,
        }
    }

    pub fn enqueue_burst<T>(&self, objs: &&mut T, len: usize) -> usize {
        unsafe {
            dpdk_sys::rte_ring_enqueue_burst(
                self.ring,
                objs as *const &mut T as *const *mut c_void,
                len as u32,
                null_mut()
            ) as usize
        }
    }

    // pub fn dequeue_burst<T>(&self, objs: &mut &mut T, len: usize) -> usize {
    pub fn dequeue_burst<T>(&self, objs: &Array<&mut T>, len: usize) -> usize {
        unsafe {
            dpdk_sys::rte_ring_dequeue_burst(
                self.ring,
                objs.as_ptr() as *mut *mut T as *mut *mut c_void,
                len as u32,
                null_mut()
            ) as usize
        }
    }

    pub fn enqueue<T>(&self, obj: &mut T) -> usize {
        unsafe {
            dpdk_sys::rte_ring_enqueue(
                self.ring,
                obj as *mut T as *mut c_void,
            ) as usize
        }
    }

    pub fn dequeue<T>(&self, obj: &mut &mut T) -> usize {
        unsafe {
            dpdk_sys::rte_ring_dequeue(
                self.ring,
                obj as *mut &mut T as *mut *mut T as *mut *mut c_void,
            ) as usize
        }
    }
}


pub struct RingBuf<T> {
    phantom: PhantomData<T>,
    mempool: *mut dpdk_sys::rte_mempool,
}

impl<T> RingBuf<T> {
    pub fn new(len: usize) -> Self {
        let mempool = unsafe {
            dpdk_sys::rte_mempool_create(
                crate::core::helper::dpdk::gen_random_name().as_ptr() as *mut c_char,
                len as u32,
                size_of::<T>() as u32,
                0,
                0,
                None,
                null_mut(),
                None,
                null_mut(),
                0,
                0
            )
        };

        RingBuf {
            phantom: PhantomData,
            mempool,
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            (*self.mempool).size as usize
        }
    }

    pub fn malloc_bulk(&self, obj: &mut [&mut T], len: usize) {
        unsafe {
            let obj_ptr = obj as *mut [&mut T] as *mut *const T;
            dpdk_sys::rte_mempool_get_bulk(self.mempool, obj_ptr as *mut *mut c_void, len as u32);
        }
    }

    pub fn free_bulk(&self, obj: &[&mut T], len: usize) {
        unsafe {
            let obj_ptr = obj as *const [&mut T] as *const *mut T;
            dpdk_sys::rte_mempool_put_bulk(self.mempool, obj_ptr as *const *mut c_void, len as u32);
        }
    }

    pub fn malloc<'a>(&'a self) -> &'a mut  T {
        unsafe {
            let mut obj_ptr: *mut T = null_mut();
            dpdk_sys::rte_mempool_get(self.mempool, &mut obj_ptr as *mut *mut T as *mut *mut c_void);
            &mut *obj_ptr as &mut T
        }
    }

    pub fn free(&self, obj: &mut T) {
        unsafe {
            dpdk_sys::rte_mempool_put(self.mempool, obj as *mut T as *mut c_void);
        }
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
