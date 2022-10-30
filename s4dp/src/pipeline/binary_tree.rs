use crate::core::memory::array::Array;
use crate::pipeline::table::FlowEntry;

pub struct BinaryTree<'a> {
    root: Option<&'a Node<'a>>,
}

impl<'a> BinaryTree<'a> {
    pub fn new(root: &Node) -> Self {
        BinaryTree {
            root: None,
        }
    }

    pub fn insert(node: &Node) {

    }
}


struct Node<'a> {
    key: u64,
    entry: &'a FlowEntry,
    next: Option<&'a Node<'a>>,
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
}

impl<'a> Node<'a> {
    pub fn new(entry: &FlowEntry, key: u64) -> Self {
        Node {
            key,
            entry,
            next: None,
            left: None,
            right: None,
        }
    }
}
