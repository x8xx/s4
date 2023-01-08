use std::ops::Index;
use std::ops::IndexMut;
use std::marker::Send;
use std::mem::size_of;
use std::ptr::null_mut;
use std::slice::from_raw_parts_mut;
use std::os::raw::c_char;
use std::ffi::c_void;
use std::marker::PhantomData;



pub struct Array<T> {
    data: *mut T,
    memzone: *const dpdk_sys::rte_memzone,
    len: usize,
}

unsafe impl<T> Send for Array<T> {}
unsafe impl<T> Sync for Array<T> {}

impl<T> Array<T> {
    pub fn new(len: usize) -> Self {
        let (memzone, data) = if len != 0 {
            let memzone = unsafe {
                dpdk_sys::rte_memzone_reserve(
                    // crate::core::helper::dpdk::gen_random_name(),
                    crate::dpdk::common::gen_random_name().as_ptr() as *mut c_char,
                    size_of::<T>() * len,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RTE_MEMZONE_SIZE_HINT_ONLY
                )
            };
            (memzone, unsafe { (*memzone).__bindgen_anon_1.addr as *mut T })
        } else {
            (null_mut() as *const dpdk_sys::rte_memzone, null_mut() as *mut T)
        };

        Array {
            data,
            memzone,
            len,
        }
    }

    pub fn init(&mut self, index: usize,  value: T) {
        unsafe {
            std::ptr::write::<T>(self.data.offset(index as isize), value);
            // *self.data.offset(index as isize) = value; 
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.data
    }


    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &mut [T] {
        unsafe {
            from_raw_parts_mut::<T>(self.data, self.len)
        }

    }

    pub fn get(&self, index: usize) -> &mut T {
        unsafe {
            &mut *self.data.offset(index as isize)
        }
    }

    pub fn free(self) {
        unsafe {
            dpdk_sys::rte_memzone_free(self.memzone);
        }
    }

    pub fn clone(&self) -> Self {
        Array {
            data: self.data,
            memzone: self.memzone,
            len: self.len,
        }
    }
}


impl<T> Index<usize> for Array<T> {
    type Output = T;

    fn index(&self, i: usize) ->  &Self::Output {
        unsafe {
            &*self.data.offset(i as isize)
        }
    }
}

impl<T> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        unsafe {
            // &mut *self.data.offset(i as isize) as &mut T
            &mut *self.data.offset(i as isize)
        }
    }
}


#[derive(Clone, Copy)]
pub struct Ring {
    ring: *mut dpdk_sys::rte_ring,
}

unsafe impl<'a> Send for Ring {}

impl Ring {
    pub fn new(len: usize) -> Self {
        let ring = {
            unsafe {
                dpdk_sys::rte_ring_create(
                    crate::dpdk::common::gen_random_name().as_ptr() as *mut c_char,
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
            )
        }.try_into().unwrap()
    }

    // pub fn dequeue_burst<T>(&self, objs: &mut &mut T, len: usize) -> usize {
    pub fn dequeue_burst<T>(&self, objs: &Array<&mut T>, len: usize) -> usize {
        unsafe {
            dpdk_sys::rte_ring_dequeue_burst(
                self.ring,
                objs.as_ptr() as *mut *mut T as *mut *mut c_void,
                len as u32,
                null_mut()
            )
        }.try_into().unwrap()
    }

    pub fn enqueue<T>(&self, obj: &mut T) -> i32 {
        unsafe {
            dpdk_sys::rte_ring_enqueue(
                self.ring,
                obj as *mut T as *mut c_void,
            )
        }
    }

    pub fn dequeue<T>(&self, obj: &mut &mut T) -> i32 {
        unsafe {
            dpdk_sys::rte_ring_dequeue(
                self.ring,
                obj as *mut &mut T as *mut *mut T as *mut *mut c_void,
            )
        }
    }

    pub fn free(self) {
        unsafe {
            dpdk_sys::rte_ring_free(self.ring);
        }

    }
}


#[derive(Clone, Copy)]
pub struct Locker {
    ring: Ring,
}

impl  Locker {
    pub fn new() -> Self {
        Locker {
            ring: Ring::new(2),
        }
    }

    pub fn unlock(&mut self) {
        let mut null = false;
        self.ring.enqueue(&mut null);
    }

    pub fn wait(&self) -> bool {
        let mut receiver = false;
        let mut ref_receiver = &mut receiver;
        loop {
            if self.ring.dequeue::<bool>(&mut ref_receiver) == 0 {
                break;
            }
        }
        true
    }

    pub fn check(&self) -> bool {
        let mut receiver = false;
        let mut ref_receiver = &mut receiver;
        self.ring.dequeue::<bool>(&mut ref_receiver) == 0
    }

    pub fn free(self) {
        self.ring.free();
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
                crate::dpdk::common::gen_random_name().as_ptr() as *mut c_char,
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
