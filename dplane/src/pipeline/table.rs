use crate::config::DpConfigTable;
use crate::core::memory::array::Array;
use crate::parser::header::Header;
use crate::parser::parse_result::ParseResult;


type HeaderID = usize;
type FieldID= usize;
type MatchField = (HeaderID, FieldID);

pub enum MatchKind {
    Exact,
    Lpm,
}

pub struct Table {
    entries: Array<FlowEntry>,
    keys: Array<(MatchField, MatchKind)>,
    default_action: ActionSet,
    len: usize,
    header_list: Array<Header>,
}

pub struct FlowEntry {
    pub values: Array<MatchFieldValue>,
    pub priority: u8,
    pub action: ActionSet,
}

pub struct MatchFieldValue {
    pub value: Option<Array<u8>>,
    pub prefix_mask: u8,
}


#[derive(Clone)]
pub struct ActionSet {
    pub action_id: u8,
    pub action_data: Array<Array<i32>>,
}


impl Table {
    pub fn new(table_conf: &DpConfigTable, header_list: Array<Header>) -> Self {
        let mut keys = Array::<(MatchField, MatchKind)>::new(table_conf.keys.len());
        for (i, key) in table_conf.keys.iter().enumerate() {
            let match_field = (key.header_id as usize, key.field_id as usize);
            let match_kind = if key.match_kind == "lpm" {
                MatchKind::Lpm
            } else {
                MatchKind::Exact
            };
            keys.init(i, (match_field, match_kind));
        } 


        let default_action = ActionSet {
            action_id: table_conf.default_action_id as u8,
            action_data: Array::new(0),
        };
        

        Table {
            entries: Array::new(table_conf.max_size as usize),
            keys,
            default_action,
            len: 0,
            header_list,
        }
    }

    pub fn search(&self, pkt: *const u8, parse_result: &ParseResult) -> &ActionSet {
        fn lpm_check(new_entry: &MatchFieldValue, old_entry: &MatchFieldValue) -> bool {
            // new: any, old: value => false
            if new_entry.value.is_none() && !old_entry.value.is_none() {
                return false;
            // new: value, old: any => true
            } else if !new_entry.value.is_none() && old_entry.value.is_none() {
                return true;
            }

            match &new_entry.value {
                Some(new_value) => {
                    match &old_entry.value {
                        Some(old_value) => {
                            if new_value.len() > old_value.len() {
                                return true;
                            }

                            if new_entry.prefix_mask > old_entry.prefix_mask {
                                return true;
                            }
                        },
                        _ => {},
                    }

                },
                _ => {},
            }
            return false;
        }


        // search entry
        let mut result_entry: Option<&FlowEntry> = None;


        // search all entry
        for i in 0..self.len {
            let mut success_match_count = 0;

            // check all key
            for j in 0..self.keys.len() {
                let match_field = self.keys[j].0;
                let match_kind = &self.keys[j].1;

                // any check
                let value = match &self.entries[i].values[j].value {
                    Some(value) => (value),
                    // any
                    None => {
                        if let MatchKind::Lpm = match_kind {
                            match &result_entry {
                                Some(entry) => {
                                    if !lpm_check(&self.entries[i].values[j], &entry.values[j]) {
                                        break;
                                    }
                                    if !entry.values[j].value.is_none() {
                                        break;
                                    }
                                }
                                _ => {},

                            }
                        }
                        success_match_count += 1;
                        continue;
                    },
                };


                // field match check
                let match_result = self.header_list[match_field.0].fields[match_field.1].cmp_pkt(
                    pkt,
                    parse_result.header_list[match_field.0].offset,
                    value,
                    self.entries[i].values[j].prefix_mask
                );
                if !match_result {
                    println!("debug2 {}", j);
                    break;
                }


                // lpm check
                if let MatchKind::Lpm = match_kind {
                    match &result_entry {
                        Some(entry) => {
                            if !lpm_check(&self.entries[i].values[j], &entry.values[j]) {
                                break;
                            }
                        }
                        _ => {},

                    }
                }

                success_match_count += 1;
            }

            if success_match_count != self.keys.len() {
                continue;
            }


            // priority check
            match result_entry {
                Some(entry) => {
                    if entry.priority < self.entries[i].priority {
                        result_entry = Some(&self.entries[i]);
                    }
                },
                None => {
                    result_entry = Some(&self.entries[i]);
                }
            }
        }


        // return action set
        match result_entry {
            Some(entry) => {
                &entry.action
            },
            None => {
                &self.default_action
            }
        }
    }

    pub fn insert(&mut self, entry: FlowEntry) {
        self.entries[self.len] = entry;
        self.len += 1;
    }


    pub fn delete(&mut self, entry_id: usize) {

    }
}


#[cfg(test)]
mod tests {
    use super::Table;
    use super::FlowEntry;
    use super::ActionSet;
    use super::MatchFieldValue;
    use crate::config::DpConfigTable;
    use crate::config::DpConfigTableKey;
    use crate::core::memory::array::Array;
    use crate::parser::header::Header;
    use crate::parser::parse_result;


    /**
     * sample dataset 1
     */
    fn get_header_list_1() -> Array<Header> {
        let mut header_list =  Array::<Header>::new(4);
        // ethernet
        header_list.init(0, Header::new(&[48, 48, 16], &[0], &[2]));
        // IPv4
        header_list.init(1, Header::new(&[4, 4, 8, 16, 16, 3, 13, 8, 8, 16, 32, 32], &[11], &[9]));
        // TCP
        header_list.init(2, Header::new(&[16, 16, 32, 32, 4, 6, 6, 16 ,16, 16], &[0, 1], &[]));
        // UDP
        header_list.init(3, Header::new(&[16, 16, 16], &[0, 1], &[]));
        
        header_list
    }

    fn get_dp_config_table_key_1() -> Vec<DpConfigTableKey> {
        let mut keys = Vec::new();

        keys.push(DpConfigTableKey {
            match_kind: "exact".to_string(),
            header_id: 0,
            field_id: 0,
        });
        keys.push(DpConfigTableKey {
            match_kind: "lpm".to_string(),
            header_id: 1,
            field_id: 11,
        });

        keys
    }

    fn get_entries_1() -> (Array<FlowEntry>, usize) {
        let entries_len = 1;
        let mut entries = Array::<FlowEntry>::new(1000);

        // entry: 0
        entries.init(0, FlowEntry {
            values: Array::new(2),
            priority: 0,
            action: ActionSet {
                action_id: 1,
                action_data: Array::new(0),
            }
        });
        // 01:02:03:04:05:06
        let mut entry_0_value_0 = Array::<u8>::new(6);
        entry_0_value_0.init(0, 0x01);
        entry_0_value_0.init(1, 0x02);
        entry_0_value_0.init(2, 0x03);
        entry_0_value_0.init(3, 0x04);
        entry_0_value_0.init(4, 0x05);
        entry_0_value_0.init(5, 0x06);
        entries[0].values.init(0, MatchFieldValue {
            value: Some(entry_0_value_0),
            prefix_mask: 0xff,
        });
        // 192.168.0.0/24
        let mut entry_0_value_1 = Array::<u8>::new(3);
        entry_0_value_1.init(0, 192);
        entry_0_value_1.init(1, 168);
        entry_0_value_1.init(2, 0);
        entries[0].values.init(1, MatchFieldValue {
            value: Some(entry_0_value_1),
            prefix_mask: 0xff,
        });

        (entries, entries_len)
    }



    #[test]
    fn test_table_search() {
        let header_list = get_header_list_1();

        let mut table_conf = DpConfigTable {
            keys: get_dp_config_table_key_1(),
            default_action_id: 0,
            max_size: 10000,
        };

        let mut table = Table::new(&table_conf, header_list.clone());
        let (entries, entries_len) = get_entries_1();
        table.entries = entries;
        table.len = entries_len;

        let mut pkt = Array::<u8>::new(64);
        pkt[0] = 0x01;
        pkt[1] = 0x02;
        pkt[2] = 0x03;
        pkt[3] = 0x04;
        pkt[4] = 0x05;
        pkt[5] = 0x06;
        pkt[28] = 192;
        pkt[29] = 168;
        pkt[30] = 0;
        pkt[31] = 24;

        let mut parse_result = parse_result::ParseResult {
            metadata: parse_result::Metadata {
                port: 0,
            },
            hdr_size: 0,
            header_list: Array::new(4),
        };
        parse_result.header_list.init(0, parse_result::Header {
            is_valid: true,
            offset: 0,
        });
        parse_result.header_list.init(1, parse_result::Header {
            is_valid: true,
            offset: 12,
        });

        
        let action_set = table.search(pkt.as_ptr(), &parse_result);
        assert_eq!(action_set.action_id, 1);
    }

}
