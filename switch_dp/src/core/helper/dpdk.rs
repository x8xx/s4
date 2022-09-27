use uuid::Uuid;
use std::ffi::CString;
use std::os::raw::c_char;

pub fn gen_mempool_name() -> *mut c_char {
    let uuid_str = Uuid::new_v4().hyphenated().to_string();
    let name_cstr = CString::new(uuid_str).unwrap();
    name_cstr.as_ptr() as *mut c_char
}
