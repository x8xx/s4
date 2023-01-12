use crate::config::DpConfigTable;
use crate::core::memory::array::Array;
use crate::core::memory::ptr::Ptr;
use crate::parser::header::Header;
use crate::parser::parse_result::ParseResult;
use crate::cache::cache::CacheRelation;
use crate::pipeline::tree::radix_tree::RadixTree;
use crate::pipeline::tree::avl_tree::AvlTree;

type HeaderID = usize;
type FieldID= usize;
type MatchField = (HeaderID, FieldID);

pub enum MatchKind {
    Exact,
    Lpm,
}

pub enum Tree {
    Radix(RadixTree),
    Avl(AvlTree),
}


pub struct Table {
    pub tree: Tree,
    pub tree_key_index: usize,

    pub keys: Array<(MatchField, MatchKind)>,
    pub default_entry: FlowEntry,
    pub header_list: Array<Header>,
}

#[derive(Clone, Copy)]
pub struct FlowEntry {
    pub values: Array<MatchFieldValue>,
    pub priority: u8,
    pub action: ActionSet,
    // pub cache_relation: Option<CacheRelation>,
}

#[derive(Clone, Copy)]
pub struct MatchFieldValue {
    pub value: Option<Array<u8>>,
    pub prefix_mask: u8,
}


#[derive(Clone, Copy)]
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


        let mut tree = if table_conf.keys[0].match_kind == "lpm" {
            Tree::Radix(RadixTree::new(0))
        } else {
            Tree::Avl(AvlTree::new(0))
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

            if i == table_conf.tree_key_index {
                tree = if table_conf.keys[i].match_kind == "lpm" {
                    Tree::Radix(RadixTree::new(i))
                } else {
                    Tree::Avl(AvlTree::new(i))
                };
            }
        } 


        Table {
            tree,
            tree_key_index: table_conf.tree_key_index,
            keys,
            default_entry,
            header_list,
        }
    }


    pub fn search(&self, pkt: *const u8, parse_result: &ParseResult) -> &FlowEntry {
        let entries = {
            let header_id = self.keys[self.tree_key_index].0.0;
            let field_id = self.keys[self.tree_key_index].0.1;
            let field = self.header_list[header_id].fields[field_id];

            let field_offset = parse_result.header_list[header_id].offset as usize + field.start_byte_pos;
            let field_len = field.end_byte_pos - field.start_byte_pos + 1;
            match &self.tree {
                Tree::Radix(tree) => {
                    tree.search(unsafe { pkt.offset(field_offset as isize) }, field_len)
                },
                Tree::Avl(tree) => {
                    tree.search(unsafe { pkt.offset(field_offset as isize) }, field_len)
                }
            }
        };

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
        for i in 0..entries.len() {
            let mut success_match_count = 0;

            // check all key
            for j in 0..self.keys.len() {
                // skip tree key
                if self.tree_key_index == j {
                    continue;
                }

                let match_field = &self.keys[j].0;
                let match_kind = &self.keys[j].1;

                // any check
                let value = match &entries[i].values[j].value {
                    Some(value) => {
                        value
                    },
                    // any
                    None => {
                        if let MatchKind::Lpm = match_kind {
                            if !lpm_check(&entries[i].values[j], &result_entry.values[j]) {
                                break;
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
                    entries[i].values[j].prefix_mask
                );
                if !match_result {
                    break;
                }


                // lpm check
                if let MatchKind::Lpm = match_kind {
                    if !lpm_check(&entries[i].values[j], &result_entry.values[j]) {
                        break;
                    }
                }

                success_match_count += 1;
            }

            if success_match_count != (self.keys.len() - 1) {
                continue;
            }


            // priority check
            if result_entry.priority <= entries[i].priority {
                result_entry = &entries[i];
            }
        }

        // return flow_entry
        &result_entry
    }


    pub fn insert(&mut self, entry: FlowEntry) {
        let entry = Ptr::new(entry);

        match &mut self.tree {
            Tree::Radix(tree) => {
                tree.add(entry);
            },
            Tree::Avl(tree) => {
                tree.add(entry);
            },
        }
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
    use crate::core::helper::linux;
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


        for i in 2..500 {
            {
                let index = i;
                entries.init(index, FlowEntry {
                    values: Array::new(entry_value_len),
                    priority: 0,
                    action: ActionSet {
                        action_id: 10,
                        action_data: Array::new(0),
                    }
                });
                entries[index].values.init(0, MatchFieldValue {
                    value: None,
                    prefix_mask: 0xff,
                });
                // any
                entries[index].values.init(1, MatchFieldValue {
                    value: None,
                    prefix_mask: 0xff,
                });
            }
        }

        let entries_len = 500;
        // let entries_len = 2;
        (entries, entries_len)
    }



    #[test]
    fn test_table_search() {
        linux::init();

        let header_list = get_header_list_1();

        let mut table_conf = DpConfigTable {
            keys: get_dp_config_table_key_1(),
            tree_key_index: 0,
            default_action_id: 0,
            max_size: 10000,
        };

        let mut table = Table::new(&table_conf, header_list.clone());
        let (entries, entries_len) = get_entries_1();
        for i in 0..entries_len {
            table.insert(entries[i]);
        }
        // table.entries = entries;
        // table.len = entries_len;


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
        assert_eq!(flow_entry.action.action_id, 10);
    }

}
