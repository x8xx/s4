use std::ffi::c_void;


#[repr(C)]
pub struct FibStartArgs {

}


pub extern "C" fn fib_start(_: *mut c_void) -> i32 {
    println!("fib_start");
    0
}
