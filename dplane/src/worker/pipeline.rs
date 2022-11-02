use std::ffi::c_void;
use std::mem::transmute;
use crate::pipeline::pipeline::Pipeline;

#[repr(C)]
pub struct PipelineArgs<'a> {
    pub pipeline: Pipeline<'a>,
}


pub extern "C" fn start_pipeline(pipeline_args_ptr: *mut c_void) -> i32 {
    println!("Start Pipeline Core");
    let pipeline_args = unsafe { &mut *transmute::<*mut c_void, *mut PipelineArgs>(pipeline_args_ptr) };
    0
}
