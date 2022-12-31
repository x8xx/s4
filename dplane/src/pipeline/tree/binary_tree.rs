use std::marker::Copy;
use std::ptr::null_mut;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::pipeline::table::FlowEntry;


pub struct BinaryTree {
    root: *mut Node,
    nodes: Vector<Node>,
    any_entries: Vector<FlowEntry>,
    key_index: usize,
}


#[derive(Clone, Copy)]
struct Node {
    left: *mut Node,
    right: *mut Node,
    entries: Vector<FlowEntry>,
    value:  Array<u8>,
}


impl BinaryTree {
    pub fn new(key_index: usize) -> Self {
        let mut nodes = Vector::new(65535, 65535);
        nodes.push(Node {
            left: null_mut(),
            right: null_mut(),
            entries: Vector::new(0, 255),
            value: Array::new(0),
        });

        BinaryTree {
            root: &mut nodes[0] as *mut Node,
            nodes,
            any_entries: Vector::new(255, 255),
            key_index,
        }
    }


    pub fn search(&self, pkt: *mut u8, len: isize) -> &Vector<FlowEntry> {
        let mut node = self.root;

        unsafe {
            loop {
                let mut is_equal = true;
                for i in 0..len {
                    if *pkt.offset(i) > (*node).value[i as usize] {
                        if (*node).right == null_mut() {
                            return &self.any_entries;
                        }
                        is_equal = false;
                        node = (*node).right;
                        break;
                    } else if *pkt.offset(i)  < (*node).value[i as usize] {
                        if (*node).left == null_mut() {
                            return &self.any_entries;
                        }
                        is_equal = false;
                        node = (*node).left;
                        break;
                    }
                }

                if is_equal {
                    return &(*node).entries;
                }
            }
        }
    }


    pub fn init_root(&self, entry: FlowEntry) -> bool {
        let value = entry.values[self.key_index].value;
        if value.is_none() {
            return false;
        }
        let value = value.unwrap();
        self.nodes.push(Node {
            left: null_mut(),
            right: null_mut(),
            entries: Vector::new(255, 255),
            value,
        });
        self.root = &mut self.nodes[0] as *mut Node;
        unsafe {
            (*self.root).entries.push(entry);
        }
        true
    }

    
    pub fn add(&self, entry: FlowEntry) {
        let value = entry.values[self.key_index].value;
        if value.is_none() {
            self.any_entries.push(entry);
            return;
        }
        let value = value.unwrap();

        let mut node = self.root;

        unsafe {
            loop {
                let mut is_equal = true;
                for i in 0..len {
                    if *pkt.offset(i) > (*node).value[i as usize] {
                        if (*node).right == null_mut() {
                            return &self.any_entries;
                        }
                        is_equal = false;
                        node = (*node).right;
                        break;
                    } else if *pkt.offset(i)  < (*node).value[i as usize] {
                        if (*node).left == null_mut() {
                            return &self.any_entries;
                        }
                        is_equal = false;
                        node = (*node).left;
                        break;
                    }
                }

                if is_equal {
                    return &(*node).entries;
                }
            }
        }

    }
}
