/**
 * metadata_id
 * 0: output_port
 */

extern {
    pub fn debug(id: i64);
    pub fn search_table(table_list_ptr: i64, table_index: i32, pkt_ptr: i64, parse_result_ptr: i64) -> i64;
    pub fn write_pkt(pkt_ptr: i64, offset: u8, value: u8);
    pub fn read_pkt(pkt_ptr: i64, offset: u8) -> u8;
    pub fn get_metadata(parse_result_ptr: i64, metadata_id: i32) -> i32;
    pub fn set_metadata(parse_result_ptr: i64, metadata_id: i32, value: i64);
    pub fn get_action_id(action_set_ptr: i64) -> i32;
    pub fn get_action_data(action_set_ptr: i64, index: i32, offset: i32) -> i32;
    pub fn to_controller(pkt_ptr: i64, parse_result_ptr: i64);
    pub fn drop(parse_result_ptr: i64);
}
