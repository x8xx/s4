use crate::core::memory::array::Array;
use crate::pipeline::table::FlowEntry;

pub struct BinaryTree<'a> {
    nodes: Array<Node<'a>>,
    root: Option<&'a Node<'a>>,
}

impl<'a> BinaryTree<'a> {
    pub fn new(len: usize) -> Self {
        let nodes = Array::new(len);
        BinaryTree {
            nodes,
            root: None,
        }
    }

    pub fn search(&self, key: u64) {

    }

    // pub fn insert(&self, node: &Node) {

    // }
}


struct Node<'a> {
    key: u64,
    entry: &'a FlowEntry,
    next: Option<&'a Node<'a>>,
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
}

impl<'a> Node<'a> {
    pub fn new(entry: &'a FlowEntry, key: u64) -> Self {
        Node {
            key,
            entry,
            next: None,
            left: None,
            right: None,
        }
    }
}