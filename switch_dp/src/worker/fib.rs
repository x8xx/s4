use std::ffi::c_void;

pub extern "C" fn fib_start(_: *mut c_void) -> i32 {
    println!("fib_start");
    0
}
