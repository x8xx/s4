use std::ffi::c_void;

pub extern "C" fn rx_start(_: *mut c_void) -> i32 {
    println!("rx_start");
    0
}
