// TODO

use std::ffi::c_void;
use std::thread;

pub fn spawn(func: extern "C" fn(*mut c_void) -> i32, args: *mut c_void) -> bool {
    true
}
