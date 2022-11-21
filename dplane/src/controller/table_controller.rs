use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::pipeline::table::Table;
use crate::pipeline::table::FlowEntry;
use crate::pipeline::table::MatchFieldValue;
use crate::pipeline::table::ActionSet;


/**
 * request buffer rule
 * value_count | ( value_len | value* | prefix_mask )* | priority | action_id | ( action_count | action_data* )* 
 */
pub fn add_flow_entry(table: &mut RwLock<Table>, request_buffer: &[u8]) {
    // create FlowEntry
    // 1. values
    let value_count = request_buffer[0];
    let mut values = Array::<MatchFieldValue>::new(value_count as usize);
    let mut pos: usize = 1;
    for i in 0..value_count as usize {
        let value_len = request_buffer[pos];
        values[i] = MatchFieldValue {
            value: if value_len == 0 {
                None
            } else {
                Some(Array::<u8>::new(value_len as usize))
            },
            prefix_mask: 0,
        };
        pos += 1;

        values[i].prefix_mask = match &mut values[i].value {
            Some(buf) => {
                for j in 0..value_len as usize {
                    buf[j] = request_buffer[pos];
                    pos += 1;
                }

                let prefix_mask = request_buffer[pos];
                pos += 1;
                prefix_mask
            },
            None => {0}
        };
    }

    // 2, priority
    let priority = request_buffer[pos];
    pos += 1;

    // 3, ActionSet
    let action_id = request_buffer[pos];
    pos += 1;

    let action_count = request_buffer[pos];
    pos += 1;
    let mut action_data = Array::<Array<i32>>::new(action_count as usize);
    for i in 0..action_count as usize {
        let action_u8_len = request_buffer[pos];
        let action_i32_len = (action_u8_len / 4) as usize;
        action_data[i] = Array::<i32>::new(action_i32_len);
        pos += 1;

        for j in 0..action_i32_len {
            action_data[i][j] = request_buffer[pos] as i32; 
            pos += 1;
            action_data[i][j] += (request_buffer[pos] as i32) << 8; 
            pos += 1;
            action_data[i][j] += (request_buffer[pos] as i32) << 16; 
            pos += 1;
            action_data[i][j] += (request_buffer[pos] as i32) << 24; 
            pos += 1;
        }

        for j in 0..(action_u8_len % 4) {
            action_data[i][action_i32_len] += (request_buffer[pos] as i32) << 8 * j;
            pos += 1;
        }
    }

    
    let flow_entry = FlowEntry {
        values,
        priority,
        action: ActionSet {
            action_id,
            action_data,
        }
    };

    let mut table = table.write().unwrap();
    table.insert(flow_entry);
}


pub fn show_flow_entry(table: &Table) -> String {
    "".to_string()
}
