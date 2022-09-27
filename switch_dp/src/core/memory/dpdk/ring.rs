
pub struct Ring {
    #[cfg(feature="dpdk")]
    ring: *mut dpdk_sys::rte_ring,
}

impl Ring {
    pub fn new() -> Self {
        #[cfg(feature="dpdk")]
        let ring = {
            unsafe {
                dpdk_sys::rte_ring_create(
                    crate::core::memory::gen_mempool_name(),
                    1024,
                    dpdk_sys::rte_socket_id() as i32,
                    dpdk_sys::RING_F_SP_ENQ | dpdk_sys::RING_F_SC_DEQ
                )
            }
        };

        Ring {
            ring,
        }
    }

    pub fn enqueue<T>(&self, obj: *mut T) -> bool {
        true
    }

    pub fn dequeue() -> bool {
        true
    }
}
