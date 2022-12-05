use std::sync::RwLock;
use crate::core::memory::array::Array;
use crate::pipeline::table::Table;
use crate::pipeline::table::FlowEntry;
use crate::pipeline::table::MatchFieldValue;
use crate::pipeline::table::ActionSet;


/**
 * request buffer rule
 * value_count | ( value_len | value* | prefix_mask )* | priority | action_id | ( action_data_len | action_data* )* 
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

    let action_data_len = request_buffer[pos];
    let action_i32_len = (action_data_len / 4) as usize;
    let action_i32_residue = (action_data_len % 4) as usize;
    pos += 1;

    let mut action_data = Array::<i32>::new(action_i32_len + if action_i32_residue == 0 {0} else {1});
    for i in 0..action_i32_len {
        action_data[i] = request_buffer[pos] as i32; 
        pos += 1;
        action_data[i] += (request_buffer[pos] as i32) << 8; 
        pos += 1;
        action_data[i] += (request_buffer[pos] as i32) << 16; 
        pos += 1;
        action_data[i] += (request_buffer[pos] as i32) << 24; 
        pos += 1;
    }

    for i in 0..action_i32_residue {
        action_data[action_i32_len] += (request_buffer[pos] as i32) << 8 * i;
        pos += 1;
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

#[cfg(test)]
mod tests {
    use std::sync::RwLock;
    use crate::core::memory::array::Array;
    use crate::config::DpConfigTable;
    use crate::config::DpConfigTableKey;
    use crate::parser::header::Header;
    use crate::parser::parse_result;
    use crate::pipeline::table::Table;
    use super::add_flow_entry;

    #[test]
    fn test_add_flow_entry() {
        let mut header_list =  Array::<Header>::new(4);
        header_list.init(0, Header::new(&[48, 48, 16], &[0, 1], &[]));

        let mut table_conf = DpConfigTable {
            keys: Vec::new(),
            default_action_id: 0,
            max_size: 10000,
        };
        table_conf.keys.push(DpConfigTableKey {
            match_kind: "exact".to_string(),
            header_id: 0,
            field_id: 0,
        });
        table_conf.keys.push(DpConfigTableKey {
            match_kind: "exact".to_string(),
            header_id: 1,
            field_id: 0,
        });

        let mut table = RwLock::new( Table::new(&table_conf, header_list));
        let mut parse_result = parse_result::ParseResult {
            metadata: Array::new(1),
            hdr_size: 0,
            header_list: Array::new(4),
        };
        parse_result.metadata.init(0, 0);
        parse_result.header_list.init(0, parse_result::Header {
            is_valid: true,
            offset: 0,
        });
        let mut pkt = Array::new(64);


        {
            let mut buffer = Vec::new();
            buffer.push(2); //value_count
            buffer.push(6); //value_len
            buffer.push(0x1); //value
            buffer.push(0x2);
            buffer.push(0x3);
            buffer.push(0x4);
            buffer.push(0x5);
            buffer.push(0x6);
            buffer.push(0xff); // prefix_mask
            buffer.push(0); //value_len (0 = any)
            buffer.push(2); // priority
            buffer.push(1); // action_id
            buffer.push(6); // action_data_len
            buffer.push(10); //action_data
            buffer.push(20);
            buffer.push(30);
            buffer.push(40);
            buffer.push(50);
            buffer.push(60);
            add_flow_entry(&mut table, &buffer);
            let unlock_table = table.read().unwrap();
            pkt[0] = 0x01;
            pkt[1] = 0x02;
            pkt[2] = 0x03;
            pkt[3] = 0x04;
            pkt[4] = 0x05;
            pkt[5] = 0x06;
            let flow_entry = unlock_table.search(pkt.as_ptr(), &parse_result);
            assert_eq!(flow_entry.priority, 2);
            assert_eq!(flow_entry.values[0].prefix_mask, 0xff);
            match &flow_entry.values[0].value {
                Some(value) => {
                    assert_eq!(value.len(), 6);
                    assert_eq!(value[0], 0x1);
                    assert_eq!(value[1], 0x2);
                    assert_eq!(value[2], 0x3);
                    assert_eq!(value[3], 0x4);
                    assert_eq!(value[4], 0x5);
                    assert_eq!(value[5], 0x6);
                },
                None => {
                    assert!(false);
                },
            }
            assert!(flow_entry.values[1].value.is_none());
            assert_eq!(flow_entry.action.action_id, 1);
            assert_eq!(flow_entry.action.action_data[0], (40 << 24) + (30 << 16) + (20 << 8) + 10);
            assert_eq!(flow_entry.action.action_data[1], (60 << 8) + 50);

        }

        
    }
}
