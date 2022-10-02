use std::mem::size_of;
use std::ptr::null_mut;

struct Buf<T> {
    mempool: *mut dpdk_sys::rte_mempool,
}

impl<T> Buf<T> {
    pub fn new(len: usize) -> Self {
        let mempool = unsafe {
                dpdk_sys::rte_mempool_create(
                crate::core::helper::dpdk::gen_random_name(),
                len,
                size_of::<T>,
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

        Buf {
            mempool,
        }
    }

}
