use std::ffi::c_void;
use std::mem::transmute;


#[repr(C)]
pub struct RxArgs {

}


pub extern "C" fn start_rx(rx_args_ptr: *mut c_void) -> i32 {
    0
}
