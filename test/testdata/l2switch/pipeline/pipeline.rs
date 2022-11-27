#![no_main]
mod libpipeline;
use libpipeline::*;


#[no_mangle]
pub fn run_pipeline(pipeline_args: i64) {
    unsafe {
        let action_set_ptr = s4_sys_search_table(pipeline_args, 0);
        let action_id = s4_sys_get_action_id(action_set_ptr);
        match action_id {
            // output x port
            0 => {
                let port = s4_sys_get_action_data(action_set_ptr, 0);
                s4_sys_set_metadata(pipeline_args, 0, port as i64);
            },
            // flooding
            1 => {
                s4_sys_flooding(pipeline_args); 
            },
            // drop
            _ => {
                s4_sys_drop(pipeline_args); 
            }
        }
    }
}
