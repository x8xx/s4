use std::marker::Copy;
use std::ptr::null_mut;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::pipeline::table::FlowEntry;


pub struct AvlTree {
    root: *mut Node,
    nodes: Vector<Node>,
    any_entries: Vector<FlowEntry>,
    key_index: usize,
}


#[derive(Clone, Copy)]
struct Node {
    parent: *mut Node,
    left: *mut Node,
    right: *mut Node,
    entries: Vector<FlowEntry>,
    value:  Array<u8>,
    height: u64,
}


impl AvlTree {
    pub fn new(key_index: usize) -> Self {
        let mut nodes = Vector::new(65535, 65535);
        nodes.push(Node {
            parent: null_mut(),
            left: null_mut(),
            right: null_mut(),
            entries: Vector::new(0, 255),
            value: Array::new(0),
            height: 0,
        });

        AvlTree {
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


    pub fn init_root(&mut self, entry: FlowEntry) -> bool {
        let value = entry.values[self.key_index].value;
        if value.is_none() {
            return false;
        }
        let value = value.unwrap();
        self.nodes.push(Node {
            parent: null_mut(),
            left: null_mut(),
            right: null_mut(),
            entries: Vector::new(255, 255),
            value,
            height: 0,
        });
        self.root = self.nodes.last() as *mut Node;
        unsafe {
            (*self.root).entries.push(entry);
        }
        true
    }

    
    pub fn add(&mut self, entry: FlowEntry) {
        fn add(tree: &mut AvlTree, entry: FlowEntry) -> Option<*mut Node> {
            let value = entry.values[tree.key_index].value;
            if value.is_none() {
                tree.any_entries.push(entry);
                return None;
            }
            let value = value.unwrap();


            let mut node = tree.root;

            unsafe {
                loop {
                    let node_value = (*node).entries[0].values[tree.key_index].value.unwrap();
                    let mut is_equal = true;

                    for i in 0..value.len() {
                        if value[i] > node_value[i] {
                            if (*node).right == null_mut() {
                                tree.nodes.push(Node {
                                    parent: null_mut(),
                                    left: null_mut(),
                                    right: null_mut(),
                                    entries: Vector::new(255, 255),
                                    value,
                                    height: (*node).height + 1,
                                });
                                (*node).right = tree.nodes.last() as *mut Node;
                                (*(*node).right).entries.push(entry);
                                return Some((*node).right);
                            }
                            is_equal = false;
                            node = (*node).right;
                            break;
                        } else if value[i] < node_value[i] {
                            if (*node).left == null_mut() {
                                tree.nodes.push(Node {
                                    parent: null_mut(),
                                    left: null_mut(),
                                    right: null_mut(),
                                    entries: Vector::new(255, 255),
                                    value,
                                    height: (*node).height + 1,
                                });
                                (*node).left = tree.nodes.last() as *mut Node;
                                (*(*node).left).entries.push(entry);
                                return Some((*node).left);
                            }
                            is_equal = false;
                            node = (*node).left;
                            break;
                        }
                    }

                    if is_equal {
                        (*node).entries.push(entry);
                        return None;
                    }
                }
            }
        }


        let node = add(self, entry);
        return;

        // if node.is_none() {
        //     return;
        // }
        // let node = node.unwrap();


        // let height_check_node = unsafe {
        //     if (*node).parent == null_mut() ||
        //         (*(*node).parent).parent == null_mut() ||
        //         (*(*(*node).parent).parent).parent == null_mut() {
        //         return;
        //     }
        //     (*(*(*node).parent).parent).parent
        // };

        // unsafe {
        //     if (*height_check_node).left == null_mut() 
        // }
    }
}


#[cfg(test)]
mod tests {
    use crate::core::memory::array::Array;
    use super::AvlTree;
    use crate::pipeline::table::FlowEntry;
    use crate::pipeline::table::ActionSet;
    use crate::pipeline::table::MatchFieldValue;

    #[test]
    fn test_avl_tree() {
        let mut tree = AvlTree::new(0);

        // any
        let action_set = ActionSet {
            action_id: 0,
            action_data: Array::new(0),
        };
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: None,
            prefix_mask: 0x00, 
        };
        tree.add(entry);
        assert_eq!(tree.any_entries.len(), 1);

        // any
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: None,
            prefix_mask: 0x00, 
        };
        tree.add(entry);
        assert_eq!(tree.any_entries.len(), 2);

        // init root node
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: None,
            prefix_mask: 0x00, 
        };
        assert!(!tree.init_root(entry));

        let action_set = ActionSet {
            action_id: 1,
            action_data: Array::new(0),
        };
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry.values[0].value.unwrap().init(0, 0x80);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        assert!(tree.init_root(entry));


        // init pkt
        let mut pkt: Vec<u8> = Vec::new();
        pkt.push(0x80);
        pkt.push(0x10);
        pkt.push(0);
        let pkt_ptr = pkt.as_ptr() as *mut u8;

        // 0x80, 0x10, 0
        assert_eq!(tree.search(pkt_ptr, 3).len(), 1);

        // 0x60, 0x10, 0
        let action_set = ActionSet {
            action_id: 2,
            action_data: Array::new(0),
        };
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry.values[0].value.unwrap().init(0, 0x60);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0x60;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        assert_eq!(tree.search(pkt_ptr, 3).len(), 1);

        // 0x60, 0x10, 0
        let action_set = ActionSet {
            action_id: 3,
            action_data: Array::new(0),
        };
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry.values[0].value.unwrap().init(0, 0x60);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].action.action_id, 2);
        assert_eq!(entries[1].action.action_id, 3);

        // 0xff, 0x10, 0
        let action_set = ActionSet {
            action_id: 4,
            action_data: Array::new(0),
        };
        let mut entry = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry.values[0].value.unwrap().init(0, 0xff);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0xff;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 4);
    }
}
