use crate::core::memory::array::Array;
use crate::pipeline::table::FlowEntry;

pub struct RadixTree<'a> {
    nodes: Array<Node<'a>>,
    // root: &'a Node<'a>,
    root: Option<&'a Node<'a>>,
}

impl <'a> RadixTree<'a> {
    pub fn new(len: usize) -> Self {
        let nodes = Array::new(len);
        // let root = &nodes[0];
        RadixTree {
            nodes,
            root: None,
        }
    }
    
    // pub fn search(value: Array<u8>) -> &'a FlowEntry {

    // }

    pub fn insert(&mut self, table_index: usize, entry: &FlowEntry) {
        let new_node = Node {
            entry: Some(entry),
            next: None,
            left: None,
            right: None,
        };

        // let value = entry.value;
        // let prefix = entry.prefix;
        // let mut target_node = self.root;

        // let mask_value = [128, 64, 32, 16, 8, 4, 2, 1];
        // let mut current_prefix = 0;

        // for i in 0..value.len() {
        //     let octet = value[i];
        //     for j in 0..8 {
        //         let bit = (octet & mask_value[j]) >> (7 - j);
        //         if bit == 1 {
        //             match target_node.right {
        //                 Some(node) => {
        //                     target_node = node;
        //                 },
        //                 None => {

        //                 },
        //             }

        //         } else {

        //         }

        //         if prefix != current_prefix {
        //             current_prefix += 1;
        //         } else {
        //             return;
        //         }
        //     }
        // }

        // self.nodes.init(table_index);
    }
}

struct Node<'a> {
    entry: Option<&'a FlowEntry>,
    next: Option<&'a Node<'a>>,
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
}
