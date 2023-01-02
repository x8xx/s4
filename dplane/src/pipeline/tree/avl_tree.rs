use std::marker::Copy;
use std::ptr::null_mut;
use crate::core::memory::heap::Heap;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::pipeline::table::FlowEntry;


pub struct AvlTree {
    root: Option<usize>,
    nodes: Vector<Node>,
    any_entries: Vector<FlowEntry>,
    key_index: usize,
    heap: Heap,
}


#[derive(Clone, Copy)]
struct Node {
    id: usize,
    parent: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    entries: Vector<FlowEntry>,
    value:  Array<u8>,
    r_height: i64,
    l_height: i64,
}


impl AvlTree {
    pub fn new(key_index: usize) -> Self {
        let nodes = Vector::new(65535, 65535);

        AvlTree {
            root: None,
            nodes,
            any_entries: Vector::new(255, 255),
            key_index,
            heap: Heap::new(536870912),
        }
    }

    
    pub fn search(&self, pkt: *const u8, len: usize) -> &Vector<FlowEntry> {
        let mut node = &self.nodes[self.root.unwrap()];

        loop {
            let mut is_equal = true;
            for i in 0..len as isize {
                if unsafe { *pkt.offset(i) } > node.value[i as usize] {
                    if node.right.is_none() {
                        return &self.any_entries;
                    }
                    is_equal = false;
                    node = &self.nodes[node.right.unwrap()];
                    break;
                } else if unsafe { *pkt.offset(i) } < node.value[i as usize] {
                    if node.left.is_none() {
                        return &self.any_entries;
                    }
                    is_equal = false;
                    node = &self.nodes[node.left.unwrap()];
                    break;
                }
            }

            if is_equal {
                return &node.entries;
            }
        }
    }


    pub fn add(&mut self, entry: FlowEntry) {
        if self.root.is_none() {
            let value = entry.values[self.key_index].value;
            if value.is_none() {
                self.any_entries.push(entry);
            } else {
                let value = value.unwrap();

                self.nodes.push(Node {
                    id: self.nodes.len(),
                    parent: None,
                    left: None,
                    right: None,
                    // entries: Vector::new(255, 255),
                    entries: self.heap.vec_malloc(16, 32),
                    value,
                    r_height: 0,
                    l_height: 0,
                });
                self.root = Some(self.nodes.len() - 1);
                self.nodes[self.root.unwrap()].entries.push(entry);
            }
            return;
        }


        let value = entry.values[self.key_index].value;
        if value.is_none() {
            self.any_entries.push(entry);
            return;
        }
        let value = value.unwrap();

        let nodes_ptr = &mut self.nodes as *mut Vector<Node>;

        unsafe {
            let mut node = (*nodes_ptr).get(self.root.unwrap());

            loop {
                let node_value = node.entries[0].values[self.key_index].value.unwrap();
                let mut is_equal = true;
                
                for i in 0..value.len() {
                    if value[i] > node_value[i] {
                        if node.right.is_none() {
                            self.nodes.push(Node {
                                id: self.nodes.len(),
                                parent: Some(node.id),
                                left: None,
                                right: None,
                                // entries: Vector::new(255, 255),
                                entries: self.heap.vec_malloc(16, 32),
                                value,
                                r_height: 0,
                                l_height: 0,
                            });
                            // node.r_height += 1;
                            node.right = Some(self.nodes.len() - 1);
                            (*nodes_ptr).get(node.right.unwrap()).entries.push(entry);
                            self.rotation(node.right.unwrap());
                            // return Some(&self.nodes[node.right.unwrap()]);
                            return;
                        }
                        is_equal = false;
                        node = (*nodes_ptr).get(node.right.unwrap());
                        break;
                    } else if value[i] < node_value[i] {
                        if node.left.is_none() {
                            self.nodes.push(Node {
                                id: self.nodes.len(),
                                parent: Some(node.id),
                                left: None,
                                right: None,
                                // entries: Vector::new(255, 255),
                                entries: self.heap.vec_malloc(16, 32),
                                value,
                                r_height: 0,
                                l_height: 0,
                            });
                            // node.l_height += 1;
                            node.left = Some(self.nodes.len() - 1);
                            self.nodes[node.left.unwrap()].entries.push(entry);
                            // return Some(&tree.nodes[node.left.unwrap()]);
                            self.rotation(node.left.unwrap());
                            return;
                        }
                        is_equal = false;
                        node = (*nodes_ptr).get(node.left.unwrap());
                        break;
                    }
                }

                if is_equal {
                    node.entries.push(entry);
                    return;
                }
            }
        }

    }


    fn rotation(&mut self, start: usize) {
        let nodes_ptr = &mut self.nodes as *mut Vector<Node>;

        unsafe {
            let mut parent = (*nodes_ptr).get(self.nodes[start].parent.unwrap());
            if !parent.right.is_none() && parent.right.unwrap() == start {
                parent.r_height += 1;
            } else {
                parent.l_height += 1;
            }
            
            loop {
                if parent.parent.is_none() {
                    self.root = Some(parent.id);
                    return;
                }
                let child_id = parent.id;
                let child_height = if parent.l_height > parent.r_height {
                    parent.l_height + 1
                } else if parent.l_height < parent.r_height {
                    parent.r_height + 1
                } else {
                    parent.r_height + 1
                };
                parent = (*nodes_ptr).get(parent.parent.unwrap());

                if !parent.right.is_none() && parent.right.unwrap() == child_id {
                    parent.r_height = child_height;
                } else {
                    parent.l_height = child_height;
                }

                let diff = parent.l_height - parent.r_height;
                if diff >= 2 {
                    let next_parent = (*nodes_ptr).get(parent.left.unwrap());
                    self.r_rotation(parent.id);
                    parent = next_parent;
                } else if diff <= -2 {
                    let next_parent = (*nodes_ptr).get(parent.right.unwrap());
                    self.l_rotation(parent.id);
                    parent = next_parent;
                }
            }
        }
    }


    fn r_rotation(&mut self, start: usize) {
        let nodes_ptr = &mut self.nodes as *mut Vector<Node>;

        unsafe {
            let root = (*nodes_ptr).get(start);
            let new_root = (*nodes_ptr).get(root.left.unwrap());
            match new_root.right {
                Some(id) => {
                    new_root.right = Some(root.id);
                    let tmp_height = new_root.r_height;
                    new_root.r_height = root.r_height + 1;
                    root.l_height = tmp_height;
                    root.left = Some(id);
                },
                None => {
                    new_root.right = Some(root.id);
                    new_root.r_height = root.r_height + 1;
                    root.l_height = 0;
                }
            }


            if root.parent.is_none() {
                root.parent = Some(new_root.id);
                new_root.parent = None;
                return;
            }
            let root_parent = (*nodes_ptr).get(root.parent.unwrap());
            if !root_parent.right.is_none() && root_parent.right.unwrap() == root.id {
                root_parent.right = Some(new_root.id);
            } else {
                root_parent.left = Some(new_root.id);
            }
            new_root.parent = root.parent;
            root.parent = Some(new_root.id);
        }
    }

    fn l_rotation(&mut self, start: usize) {
        let nodes_ptr = &mut self.nodes as *mut Vector<Node>;

        unsafe {
            let root = (*nodes_ptr).get(start);
            let new_root = (*nodes_ptr).get(root.right.unwrap());
            match new_root.left {
                Some(id) => {
                    new_root.left = Some(root.id);
                    let tmp_height = new_root.l_height;
                    new_root.l_height = root.l_height + 1;
                    root.r_height = tmp_height;
                    root.right = Some(id);
                },
                None => {
                    new_root.left = Some(root.id);
                    new_root.l_height = root.l_height + 1;
                    root.r_height = 0;
                }
            }


            if root.parent.is_none() {
                root.parent = Some(new_root.id);
                new_root.parent = None;
                return;
            }
            let root_parent = (*nodes_ptr).get(root.parent.unwrap());
            if !root_parent.right.is_none() && root_parent.right.unwrap() == root.id {
                root_parent.right = Some(new_root.id);
            } else {
                root_parent.left = Some(new_root.id);
            }
            new_root.parent = root.parent;
            root.parent = Some(new_root.id);
        }

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


        // 0x80, 0x10, 0
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
        // assert!(tree.init_root(entry));
        tree.add(entry);


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

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 0);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 1);

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

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 0);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 1);

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

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 1);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 1);


        // 0xff, 0x20, 0
        let action_set = ActionSet {
            action_id: 5,
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
        entry.values[0].value.unwrap().init(1, 0x20);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries[0].action.action_id, 4);
        pkt[0] = 0xff;
        pkt[1] = 0x20;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 5);


        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 2);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 1);


        // 0xff, 0x30, 0
        let action_set = ActionSet {
            action_id: 6,
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
        entry.values[0].value.unwrap().init(1, 0x30);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0xff;
        pkt[1] = 0x30;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 6);

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 2);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 1);


        // 0xff, 0x40, 0
        let action_set = ActionSet {
            action_id: 7,
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
        entry.values[0].value.unwrap().init(1, 0x40);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0xff;
        pkt[1] = 0x40;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 7);

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 2);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 2);


        // 0x50, 0x10, 0
        let action_set = ActionSet {
            action_id: 8,
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
        entry.values[0].value.unwrap().init(0, 0x50);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0x50;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 8);

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 2);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 3);


        // 0x40, 0x10, 0
        let action_set = ActionSet {
            action_id: 9,
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
        entry.values[0].value.unwrap().init(0, 0x40);
        entry.values[0].value.unwrap().init(1, 0x10);
        entry.values[0].value.unwrap().init(2, 0);
        tree.add(entry);
        pkt[0] = 0x40;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 9);

        assert_eq!(tree.nodes[tree.root.unwrap()].r_height, 2);
        assert_eq!(tree.nodes[tree.root.unwrap()].l_height, 3);
    }
}
