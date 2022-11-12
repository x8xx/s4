use std::ffi::c_void;
use std::mem::transmute;
use crate::core::memory::array::Array;
use crate::core::memory::ring::Ring;
use crate::core::network::interface::Interface;
use crate::worker::rx::RxResult;

#[repr(C)]
pub struct TxArgs {
    pub name: String,
    pub ring: Ring,
    pub batch_count: usize,
}

pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    println!("Start Tx Core");
    let tx_args = unsafe { &mut *transmute::<*mut c_void, *mut TxArgs>(tx_args_ptr) };

    let interface = Interface::new(&tx_args.name);
    let mut rx_result_list = Array::<&mut RxResult>::new(tx_args.batch_count);
    loop {
        let rx_result_dequeue_count = tx_args.ring.dequeue_burst::<RxResult>(&mut rx_result_list[0], tx_args.batch_count);

        for i in 0..rx_result_dequeue_count {
            let rx_result = &mut rx_result_list[i];

            (*rx_result).free();
            // let id = (*rx_result).id;
            // (*rx_result).pktbuf.free();
            // (*rx_result).owner_ring.free(rx_result_list[i]);
            // println!("free {}", id);
        }
    }
    0
}
