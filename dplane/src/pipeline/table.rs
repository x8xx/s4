use crate::config::DpConfigTable;
use crate::core::memory::array::Array;
use crate::parser::header::Header;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheRelation;


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
    // default_action: ActionSet,
    default_entry: FlowEntry,
    len: usize,
    header_list: Array<Header>,
}

pub struct FlowEntry {
    pub values: Array<MatchFieldValue>,
    pub priority: u8,
    pub action: ActionSet,
    // pub cache_relation: Option<CacheRelation>,
}

pub struct MatchFieldValue {
    pub value: Option<Array<u8>>,
    pub prefix_mask: u8,
}


#[derive(Clone)]
pub struct ActionSet {
    pub action_id: u8,
    pub action_data: Array<i32>,
}


impl Table {
    pub fn new(table_conf: &DpConfigTable, header_list: Array<Header>) -> Self {
        let mut keys = Array::<(MatchField, MatchKind)>::new(table_conf.keys.len());

        let mut default_entry = FlowEntry {
            values: Array::new(table_conf.keys.len()),
            priority: 0,
            action: ActionSet {
                action_id: table_conf.default_action_id as u8,
                action_data: Array::new(0),
            },
        };


        for (i, key) in table_conf.keys.iter().enumerate() {
            let match_field = (key.header_id as usize, key.field_id as usize);
            let match_kind = if key.match_kind == "lpm" {
                MatchKind::Lpm
            } else {
                MatchKind::Exact
            };
            keys.init(i, (match_field, match_kind));
            default_entry.values.init(i, MatchFieldValue {
                value: None,
                prefix_mask: 0,
            });
        } 


        Table {
            entries: Array::new(table_conf.max_size as usize),
            keys,
            default_entry,
            len: 0,
            header_list,
        }
    }


    pub fn search(&self, pkt: *const u8, parse_result: &ParseResult) -> &FlowEntry {
        fn lpm_check(new_entry: &MatchFieldValue, old_entry: &MatchFieldValue) -> bool {
            // new: any, old: value => false
            if new_entry.value.is_none() && !old_entry.value.is_none() {
                return false;
            // new: value, old: any => true
            } else if !new_entry.value.is_none() && old_entry.value.is_none() {
                return true;
            // new: any, old: any => true
            } else if new_entry.value.is_none() == old_entry.value.is_none() {
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


        // search result entry
        let mut result_entry = &self.default_entry;

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
                            if !lpm_check(&self.entries[i].values[j], &result_entry.values[j]) {
                                break;
                            }
                            // if !result_entry.values[j].value.is_none() {
                            //     break;
                            // }
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
                    break;
                }


                // lpm check
                if let MatchKind::Lpm = match_kind {
                    if !lpm_check(&self.entries[i].values[j], &result_entry.values[j]) {
                        break;
                    }
                }

                success_match_count += 1;
            }

            if success_match_count != self.keys.len() {
                continue;
            }


            // priority check
            if result_entry.priority <= self.entries[i].priority {
                result_entry = &self.entries[i];
            }
        }

        // return flow_entry
        &result_entry
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

        // dst mac address
        keys.push(DpConfigTableKey {
            match_kind: "exact".to_string(),
            header_id: 0,
            field_id: 0,
        });
        // dst IPv4 address
        keys.push(DpConfigTableKey {
            match_kind: "lpm".to_string(),
            header_id: 1,
            field_id: 11,
        });

        keys
    }

    fn get_entries_1() -> (Array<FlowEntry>, usize) {
        let mut entries = Array::<FlowEntry>::new(1000);
        let entry_value_len = 2;

        // entry: 0
        {
            let index = 0;
            entries.init(index, FlowEntry {
                values: Array::new(entry_value_len),
                priority: 0,
                action: ActionSet {
                    action_id: 1,
                    action_data: Array::new(0),
                }
            });
            // 01:02:03:04:05:06
            let mut entry_value = Array::<u8>::new(6);
            entry_value.init(0, 0x01);
            entry_value.init(1, 0x02);
            entry_value.init(2, 0x03);
            entry_value.init(3, 0x04);
            entry_value.init(4, 0x05);
            entry_value.init(5, 0x06);
            entries[index].values.init(0, MatchFieldValue {
                value: Some(entry_value),
                prefix_mask: 0xff,
            });
            // 192.168.0.0/24
            let mut entry_value = Array::<u8>::new(3);
            entry_value.init(0, 192);
            entry_value.init(1, 168);
            entry_value.init(2, 0);
            entries[index].values.init(1, MatchFieldValue {
                value: Some(entry_value),
                prefix_mask: 0xff,
            });
        }


        // entry: 1
        {
            let index = 1;
            entries.init(index, FlowEntry {
                values: Array::new(entry_value_len),
                priority: 0,
                action: ActionSet {
                    action_id: 2,
                    action_data: Array::new(0),
                }
            });
            // 01:02:03:04:05:06
            let mut entry_value = Array::<u8>::new(6);
            entry_value.init(0, 0x01);
            entry_value.init(1, 0x02);
            entry_value.init(2, 0x03);
            entry_value.init(3, 0x04);
            entry_value.init(4, 0x05);
            entry_value.init(5, 0x06);
            entries[index].values.init(0, MatchFieldValue {
                value: Some(entry_value),
                prefix_mask: 0xff,
            });
            // any
            entries[index].values.init(1, MatchFieldValue {
                value: None,
                prefix_mask: 0xff,
            });
        }


        let entries_len = 2;
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
            offset: 14,
        });


        let mut pkt = Array::<u8>::new(64);

        pkt[0] = 0x01;
        pkt[1] = 0x02;
        pkt[2] = 0x03;
        pkt[3] = 0x04;
        pkt[4] = 0x05;
        pkt[5] = 0x06;
        pkt[30] = 192;
        pkt[31] = 168;
        pkt[32] = 0;
        pkt[33] = 24;
        let mut flow_entry = table.search(pkt.as_ptr(), &parse_result);
        assert_eq!(flow_entry.action.action_id, 1);

        pkt[30] = 172;
        pkt[31] = 16;
        pkt[32] = 0;
        pkt[33] = 24;
        flow_entry = table.search(pkt.as_ptr(), &parse_result);
        assert_eq!(flow_entry.action.action_id, 2);

        pkt[5] = 0x07;
        flow_entry = table.search(pkt.as_ptr(), &parse_result);
        assert_eq!(flow_entry.action.action_id, 0);
    }

}
