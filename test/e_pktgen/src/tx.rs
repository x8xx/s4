use std::ffi::c_void;
use std::mem::transmute;


#[repr(C)]
pub struct TxArgs {
}


pub extern "C" fn start_tx(tx_args_ptr: *mut c_void) -> i32 {
    0
}
