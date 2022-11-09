use crate::config::DpConfigTable;
use crate::core::memory::array::Array;
use crate::parser::header::Header;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::binary_tree::BinaryTree;
use crate::pipeline::radix_tree::RadixTree;


type HeaderID = usize;
type FieldID= usize;
type MatchField = (HeaderID, FieldID);

pub enum MatchKind<'a> {
    Exact(MatchField, BinaryTree<'a>),
    Lpm(MatchField, RadixTree<'a>),
}

impl<'a> MatchKind<'a> {
    pub fn is_match(&self, header_list: &'a Array<Header>, parse_result: &'a ParseResult, pkt: *const u8, entry: &'a FlowEntry) -> bool {
        let value = match &entry.value {
            Some(value) => value,
            None => return true,
        };

        match self {
            MatchKind::Exact(match_field, _) => {
                header_list[match_field.0].fields[match_field.1].cmp_pkt(
                    pkt,
                    parse_result.header_list[match_field.0].offset,
                    &value,
                    entry.end_bit_mask
                )
            },
            MatchKind::Lpm(match_field, _) => {
                // header_list[match_field.0].fields[match_field.1].cmp_lpm_match(
                //     pkt,
                //     &value,
                //     parse_result.header_list[match_field.0].offset,
                //     entry.prefix
                // )
                true
            },
        }
    }

    // pub fn tree_search(&self, pkt: *const u8) -> &FlowEntry {
    //     match self {
    //         MatchKind::Exact(field, tree) => {

    //         },
    //         MatchKind::Lpm(field, tree) => {

    //         },
    //     }
    // }
}


pub struct Table<'a> {
    entries: Array<FlowEntry>,
    // delete_entry_indexes: Array<usize>,
    keys: Array<MatchKind<'a>>,
    // tree: &'a MatchKind<'a>,
    default_action: ActionSet,
    len: usize,
    tree_search_lock: bool,
    tree_edit_lock: bool,
    header_list: &'a Array<Header>,
}


pub struct FlowEntry {
    pub value: Option<Array<u8>>,
    pub end_bit_mask: u8,
    pub prefix: u8,
    pub priority: u8,
    pub is_delete: bool,
    pub action: ActionSet,
}


pub struct ActionSet {
    pub action_id: u8,
    pub action_data: Array<Array<i32>>,
}


impl<'a> Table<'a> {
    pub fn new(table_conf: &DpConfigTable, header_list: &'a Array<Header>) -> Self {
        let mut keys = Array::<MatchKind>::new(table_conf.keys.len());
        for (i, key) in table_conf.keys.iter().enumerate() {
            let match_kind = if key.match_kind == "lpm" {
                let match_field = (key.header_id as usize, key.field_id as usize);
                let tree = RadixTree::new(table_conf.max_size as usize);
                MatchKind::Lpm(match_field, tree)
            } else {
                let match_field = (key.header_id as usize, key.field_id as usize);
                let tree = BinaryTree::new(table_conf.max_size as usize);
                MatchKind::Exact(match_field, tree)
            };
            keys.init(i, match_kind);
        } 

        let default_action = ActionSet {
            action_id: table_conf.default_action_id as u8,
            action_data: Array::new(0),
        };
        

        Table {
            entries: Array::new(table_conf.max_size as usize),
            keys,
            // tree: &keys[0],
            default_action,
            len: 0,
            tree_search_lock: true,
            tree_edit_lock: true,
            header_list,
        }
    }

    pub fn search(&'a mut self, pkt: *const u8, parse_result: &'a ParseResult) -> &ActionSet {
        // search entry
        if self.tree_search_lock {
            let mut result_entry: Option<&FlowEntry> = None;

            // search all entry
            for i in 0..self.len {
                let mut success_match_count = 0;
                // check all key
                for j in 0..self.keys.len() {
                    let match_result = self.keys[j].is_match(
                        self.header_list,
                        parse_result,
                        pkt,
                        &self.entries[i],
                    );

                    if !match_result {
                        break;
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
        } else {
            self.tree_edit_lock = true;
            // let entry = self.tree.tree_search(pkt);

            // if filltered_entries.len() == 0 {
            //     return &self.default_action;
            // }


            self.tree_edit_lock = false;

            // &entry.action
            &self.default_action

            // priority check
            // let mut max_priority_entry = entries[0];
            // for i in 1..entries.len() {
            //     if entries[i].priority > max_priority_entry {
            //         max_priority_entry = entries[i];
            //     }
            // }

            // max_priority_entry
        }
    }


    pub fn insert(&mut self, entry: FlowEntry) {
        self.entries[self.len] = entry;
        self.len += 1;

        // self.tree_search_lock = true;
        // while self.tree_edit_lock {}

        // self.tree_search_lock = false;
    }


    pub fn delete(&mut self, entry_id: usize) {

    }
}
