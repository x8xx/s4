/**
 * metadata_id
 * 0: output_port
 */

extern {
    pub fn s4_sys_debug(id: i64);

    pub fn s4_sys_search_table(pipeline_args_ptr: i64, table_id: i32) -> i64;

    // pkt
    pub fn s4_sys_write_pkt(pipeline_args_ptr: i64, offset: u8, value: u8);
    pub fn s4_sys_read_pkt(pipeline_args_ptr: i64, offset: u8) -> u8;

    // metadata
    pub fn s4_sys_get_metadata(pipeline_args_ptr: i64, metadata_id: i32) -> i32;
    pub fn s4_sys_set_metadata(pipeline_args_ptr: i64, metadata_id: i32, value: i64);

    // action
    pub fn s4_sys_get_action_id(action_set_ptr: i64) -> i32;
    pub fn s4_sys_get_action_data(action_set_ptr: i64, index: i32) -> i32;

    // where to send
    pub fn s4_sys_to_controller(pipeline_args_ptr: i64);
    pub fn s4_sys_drop(pipeline_args_ptr: i64);
    pub fn s4_sys_flooding(pipeline_args_ptr: i64);
}
