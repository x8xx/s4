use crate::core::memory::array::Array;
use crate::parser::parse_result::ParseResult;
use crate::pipeline::binary_tree;
use crate::pipeline::radix_tree;

type HeaderID = usize;
type FieldID= usize;
type MatchField = (HeaderID, FieldID);

pub enum MatchKind<'a>  {
    Exact(MatchField, binary_tree::BinaryTree<'a>),
    Lpm(MatchField, radix_tree::RadixTree<'a>),
}

impl<'a> MatchKind<'a> {
    pub fn is_match(kind: MatchKind, entry_value: Option<Array<u8>>, entry_prefix: u8,  pkt: &[u8]) -> bool {
        if entry_value == None {
            return true;
        }
        let value = entry_value.unwrap();

        match kind {
            MatchKind::Exact(_) => {

            },
            MatchKind::Lpm(_) => {

            },
        }
        true
    }
}

// pub enum TreeKind<'a> {
// }


pub struct Table<'a> {
    entries: Array<FlowEntry>,
    // delete_entry_indexes: Array<usize>,
    keys: Array<MatchKind<'a>>,
    tree_search_lock: bool,
    tree_edit_lock: bool,
    len: usize,
    default_action: ActionSet,
}

impl<'a> Table<'a> {
    pub fn new(max_table_size: usize, keys: Array<MatchKind>) -> Self {
        Table {
            entries: Array::new(max_table_size),
            keys,
            tree_search_lock: true,
            tree_edit_lock: true,
            len: 0,
        }
    }

    pub fn search(&mut self, parse_result: &ParseResult, pkt: &[u8]) -> Option<&FlowEntry> {
        // search entry
        if self.tree_search_lock {
            for entry in self.entries {
                let match_count = 0;
                for key in self.keys {
                    if !MatchKind::is_match(key, entry.value, entry.prefix, pkt) {
                        break;
                    }
                    match_count += 1;
                }

                if match_count == self.keys.len() {
                }
            }
        } else {
            self.tree_edit_lock = true;
            for key in self.keys {

            }
            self.tree_edit_lock = false;
        }

        // priority check
    }

    pub fn insert(&mut self, entry: FlowEntry) {
        self.entries[self.len] = entry;
        self.len += 1;

        self.tree_search_lock = true;
        while self.tree_edit_lock {}

        self.tree_search_lock = false;
    }

    pub fn delete(&mut self, entry_id: usize) {

    }
}


pub struct FlowEntry {
    value: Array<Option<Array<u8>>>,
    prefix: u8,
    priority: u8,
    is_delete: bool,
    action: ActionSet,
}

pub struct ActionSet {
    action_id: u8,
    action_data: Array<Array<u8>>,
}


pub fn wasm_native_func_search_table(table_id: i64, pkt_id: i64, parse_result_id: i64) -> i64 {
    let table = table_id as *const Table as &Table;
    let pkt = pkt_id as *const u8;
    let parse_result = parse_result_id as *const ParseResult as &ParseResult;

    let entry = table.search(parse_result, pkt);
    let action_set = match entry {
        Some(entry) => &entry.action,
        None => &table.default_action,
    };

    action_set as *const ActionSet as i64
}
