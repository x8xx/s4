use std::ffi::c_void;

#[repr(C)]
pub struct CacheMainArgs {
}

pub struct CacheRingMetadata {

}

pub extern "C" fn cache_main(args_ptr: *mut c_void) -> i32 {
    0
}
