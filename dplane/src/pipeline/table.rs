use crate::config::DpConfigTable;
use crate::core::memory::array::Array;
use crate::parser::header::Header;
use crate::parser::parse_result::ParseResult;


type HeaderID = usize;
type FieldID= usize;
type MatchField = (HeaderID, FieldID);

pub enum MatchKind {
    Exact,
    Lpm
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
                // let match_field = (key.header_id as usize, key.field_id as usize);
                // MatchKind::Lpm(match_field)
                MatchKind::Lpm
            } else {
                // let match_field = (key.header_id as usize, key.field_id as usize);
                // MatchKind::Exact(match_field)
                MatchKind::Exact
            };
            // keys.init(i, match_kind);
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
            if new_entry.value.is_none() && !old_entry.value.is_none() {
                return false;
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
