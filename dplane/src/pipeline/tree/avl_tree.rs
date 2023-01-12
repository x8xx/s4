use crate::core::memory::ptr::Ptr;
use crate::core::memory::array::Array;
use crate::core::memory::vector::Vector;
use crate::pipeline::table::FlowEntry;


pub struct AvlTree {
    root: Option<Ptr<Node>>,
    any_entries: Vector<Ptr<FlowEntry>>,
    key_index: usize,
}

#[derive(Clone, Copy)]
struct Node {
    parent: Option<Ptr<Node>>,
    left: Option<Ptr<Node>>,
    right: Option<Ptr<Node>>,
    entries: Vector<Ptr<FlowEntry>>,
    value:  Array<u8>,
    r_height: i64,
    l_height: i64,
}


impl AvlTree {
    pub fn new(key_index: usize) -> Self {
        AvlTree {
            root: None,
            // nodes,
            any_entries: Vector::new(255, 255),
            key_index,
        }
    }

    
    pub fn search(&self, pkt: *const u8, len: usize) -> &Vector<Ptr<FlowEntry>> {
        let mut node = self.root.as_ref().unwrap();

        loop {
            let mut is_equal = true;
            for i in 0..len as isize {
                if unsafe { *pkt.offset(i) } > node.value[i as usize] {
                    if node.right.is_none() {
                        return &self.any_entries;
                    }
                    is_equal = false;
                    node = &node.right.as_ref().unwrap();
                    break;
                } else if unsafe { *pkt.offset(i) } < node.value[i as usize] {
                    if node.left.is_none() {
                        return &self.any_entries;
                    }
                    is_equal = false;
                    node = &node.left.as_ref().unwrap();
                    break;
                }
            }

            if is_equal {
                return &node.entries;
            }
        }
    }


    pub fn add(&mut self, entry: Ptr<FlowEntry>) {
        if self.root.is_none() {
            let value = entry.values[self.key_index].value;
            if value.is_none() {
                self.any_entries.push(entry);
            } else {
                let value = value.unwrap();

                self.root = Some(Ptr::new(Node {
                    // id: self.nodes.len(),
                    parent: None,
                    left: None,
                    right: None,
                    // entries: Vector::new(255, 255),
                    entries: Vector::new(8, 8),
                    value,
                    r_height: 0,
                    l_height: 0,
                }));
                // self.root = Some(self.nodes.len() - 1);
                // self.nodes[self.root.unwrap()].entries.push(entry);
                self.root.as_mut().unwrap().entries.push(entry);
            }
            return;
        }


        let value = entry.values[self.key_index].value;
        if value.is_none() {
            self.any_entries.push(entry);
            return;
        }
        let value = value.unwrap();


        let mut node = self.root.unwrap();

        loop {
            let node_value = node.entries[0].values[self.key_index].value.unwrap();
            let mut is_equal = true;

            
            for i in 0..value.len() {
                if value[i] > node_value[i] {
                    if node.right.is_none() {
                        node.right = Some(Ptr::new(Node {
                            parent: Some(node),
                            left: None,
                            right: None,
                            // entries: Vector::new(255, 255),
                            entries: Vector::new(8, 8),
                            value,
                            r_height: 0,
                            l_height: 0,
                        }));
                        node.right.unwrap().entries.push(entry);
                        self.rotation(node.right.unwrap());
                        return;
                    }
                    is_equal = false;
                    node = node.right.unwrap();
                    break;
                } else if value[i] < node_value[i] {
                    if node.left.is_none() {
                        node.left = Some(Ptr::new(Node {
                            parent: Some(node),
                            left: None,
                            right: None,
                            entries: Vector::new(8, 8),
                            value,
                            r_height: 0,
                            l_height: 0,
                        }));
                        node.left.unwrap().entries.push(entry);
                        self.rotation(node.left.unwrap());
                        return;
                    }
                    is_equal = false;
                    node = node.left.unwrap();
                    break;
                }
            }

            if is_equal {
                node.entries.push(entry);
                return;
            }
        }
        // }

    }


    fn rotation(&mut self, start: Ptr<Node>) {
        let mut parent = start.parent.unwrap();
        if !parent.right.is_none() &&  parent.right.unwrap()  == start {
            parent.r_height += 1;
        } else {
            parent.l_height += 1;
        }
        
        loop {
            if parent.parent.is_none() {
                self.root = Some(parent);
                return;
            }
            let child = parent;
            let child_height = if parent.l_height > parent.r_height {
                parent.l_height + 1
            } else if parent.l_height < parent.r_height {
                parent.r_height + 1
            } else {
                parent.r_height + 1
            };
            parent = parent.parent.unwrap();

            if !parent.right.is_none() && parent.right.unwrap() == child {
                parent.r_height = child_height;
            } else {
                parent.l_height = child_height;
            }

            let diff = parent.l_height - parent.r_height;
            if diff >= 2 {
                let next_parent = parent.left.unwrap();
                self.r_rotation(parent);
                parent = next_parent;
            } else if diff <= -2 {
                let next_parent = parent.right.unwrap();
                self.l_rotation(parent);
                parent = next_parent;
            }
        }
    }


    fn r_rotation(&mut self, start: Ptr<Node>) {
        let mut root = start;
        let mut new_root = root.left.unwrap();
        match new_root.right {
            Some(node) => {
                new_root.right = Some(root);
                let tmp_height = new_root.r_height;
                new_root.r_height = root.r_height + 1;
                root.l_height = tmp_height;
                root.left = Some(node);
            },
            None => {
                new_root.right = Some(root);
                new_root.r_height = root.r_height + 1;
                root.l_height = 0;
            }
        }


        if root.parent.is_none() {
            root.parent = Some(new_root);
            new_root.parent = None;
            return;
        }
        let mut root_parent = root.parent.unwrap();
        if !root_parent.right.is_none() && root_parent.right.unwrap() == root {
            root_parent.right = Some(new_root);
        } else {
            root_parent.left = Some(new_root);
        }
        new_root.parent = root.parent;
        root.parent = Some(new_root);
    }

    fn l_rotation(&mut self, start: Ptr<Node>) {
        let mut root = start;
        let mut new_root = root.right.unwrap();
        match new_root.left {
            Some(node) => {
                new_root.left = Some(root);
                let tmp_height = new_root.l_height;
                new_root.l_height = root.l_height + 1;
                root.r_height = tmp_height;
                root.right = Some(node);
            },
            None => {
                new_root.left = Some(root);
                new_root.l_height = root.l_height + 1;
                root.r_height = 0;
            }
        }


        if root.parent.is_none() {
            root.parent = Some(new_root);
            new_root.parent = None;
            return;
        }
        let mut root_parent = root.parent.unwrap();
        if !root_parent.right.is_none() && root_parent.right.unwrap() == root {
            root_parent.right = Some(new_root);
        } else {
            root_parent.left = Some(new_root);
        }
        new_root.parent = root.parent;
        root.parent = Some(new_root);
    }
}


#[cfg(test)]
mod tests {
    use crate::core::memory::array::Array;
    use crate::core::memory::ptr::Ptr;
    use super::AvlTree;
    use crate::core::helper::linux;
    use crate::pipeline::table::FlowEntry;
    use crate::pipeline::table::ActionSet;
    use crate::pipeline::table::MatchFieldValue;

    #[test]
    fn test_avl_tree() {
        linux::init();

        let mut tree = AvlTree::new(0);

        // any
        let action_set = ActionSet {
            action_id: 0,
            action_data: Array::new(0),
        };
        let mut entry0 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry0.values[0] = MatchFieldValue {
            value: None,
            prefix_mask: 0x00, 
        };
        tree.add(Ptr::new(entry0));
        assert_eq!(tree.any_entries.len(), 1);

        // any
        let mut entry1 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry1.values[0] = MatchFieldValue {
            value: None,
            prefix_mask: 0x00, 
        };
        tree.add(Ptr::new(entry1));
        assert_eq!(tree.any_entries.len(), 2);


        // 0x80, 0x10, 0
        let action_set = ActionSet {
            action_id: 1,
            action_data: Array::new(0),
        };
        let mut entry2 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry2.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry2.values[0].value.unwrap().init(0, 0x80);
        entry2.values[0].value.unwrap().init(1, 0x10);
        entry2.values[0].value.unwrap().init(2, 0);
        // assert!(tree.init_root(entry));
        tree.add(Ptr::new(entry2));


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
        let mut entry3 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry3.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry3.values[0].value.unwrap().init(0, 0x60);
        entry3.values[0].value.unwrap().init(1, 0x10);
        entry3.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry3));
        pkt[0] = 0x60;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        assert_eq!(tree.search(pkt_ptr, 3).len(), 1);

        assert_eq!(tree.root.unwrap().r_height, 0);
        assert_eq!(tree.root.unwrap().l_height, 1);

        // 0x60, 0x10, 0
        let action_set = ActionSet {
            action_id: 3,
            action_data: Array::new(0),
        };
        let mut entry4 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry4.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry4.values[0].value.unwrap().init(0, 0x60);
        entry4.values[0].value.unwrap().init(1, 0x10);
        entry4.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry4));
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].action.action_id, 2);
        assert_eq!(entries[1].action.action_id, 3);

        assert_eq!(tree.root.unwrap().r_height, 0);
        assert_eq!(tree.root.unwrap().l_height, 1);

        // 0xff, 0x10, 0
        let action_set = ActionSet {
            action_id: 4,
            action_data: Array::new(0),
        };
        let mut entry5 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry5.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry5.values[0].value.unwrap().init(0, 0xff);
        entry5.values[0].value.unwrap().init(1, 0x10);
        entry5.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry5));
        pkt[0] = 0xff;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 4);

        assert_eq!(tree.root.unwrap().r_height, 1);
        assert_eq!(tree.root.unwrap().l_height, 1);


        // 0xff, 0x20, 0
        let action_set = ActionSet {
            action_id: 5,
            action_data: Array::new(0),
        };
        let mut entry6 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry6.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry6.values[0].value.unwrap().init(0, 0xff);
        entry6.values[0].value.unwrap().init(1, 0x20);
        entry6.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry6));
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries[0].action.action_id, 4);
        pkt[0] = 0xff;
        pkt[1] = 0x20;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 5);


        assert_eq!(tree.root.unwrap().r_height, 2);
        assert_eq!(tree.root.unwrap().l_height, 1);


        // 0xff, 0x30, 0
        let action_set = ActionSet {
            action_id: 6,
            action_data: Array::new(0),
        };
        let mut entry7 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry7.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry7.values[0].value.unwrap().init(0, 0xff);
        entry7.values[0].value.unwrap().init(1, 0x30);
        entry7.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry7));
        pkt[0] = 0xff;
        pkt[1] = 0x30;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 6);

        assert_eq!(tree.root.unwrap().r_height, 2);
        assert_eq!(tree.root.unwrap().l_height, 1);


        // 0xff, 0x40, 0
        let action_set = ActionSet {
            action_id: 7,
            action_data: Array::new(0),
        };
        let mut entry8 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry8.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry8.values[0].value.unwrap().init(0, 0xff);
        entry8.values[0].value.unwrap().init(1, 0x40);
        entry8.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry8));
        pkt[0] = 0xff;
        pkt[1] = 0x40;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 7);

        assert_eq!(tree.root.unwrap().r_height, 2);
        assert_eq!(tree.root.unwrap().l_height, 2);


        // 0x50, 0x10, 0
        let action_set = ActionSet {
            action_id: 8,
            action_data: Array::new(0),
        };
        let mut entry9 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry9.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry9.values[0].value.unwrap().init(0, 0x50);
        entry9.values[0].value.unwrap().init(1, 0x10);
        entry9.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry9));
        pkt[0] = 0x50;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 8);

        assert_eq!(tree.root.unwrap().r_height, 2);
        assert_eq!(tree.root.unwrap().l_height, 3);


        // 0x40, 0x10, 0
        let action_set = ActionSet {
            action_id: 9,
            action_data: Array::new(0),
        };
        let mut entry10 = FlowEntry {
            values: Array::new(1),
            priority: 0,
            action: action_set,
        };
        entry10.values[0] = MatchFieldValue {
            value: Some(Array::new(3)),
            prefix_mask: 0xff, 
        };
        entry10.values[0].value.unwrap().init(0, 0x40);
        entry10.values[0].value.unwrap().init(1, 0x10);
        entry10.values[0].value.unwrap().init(2, 0);
        tree.add(Ptr::new(entry10));
        pkt[0] = 0x40;
        pkt[1] = 0x10;
        pkt[2] = 0x0;
        let entries = tree.search(pkt_ptr, 3);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action.action_id, 9);

        assert_eq!(tree.root.unwrap().r_height, 2);
        assert_eq!(tree.root.unwrap().l_height, 3);
    }
}
