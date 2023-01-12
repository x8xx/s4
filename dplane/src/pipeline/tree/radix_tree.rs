use crate::core::memory::vector::Vector;
use crate::core::memory::ptr::Ptr;
use crate::pipeline::table::FlowEntry;


pub struct RadixTree {
    root: Ptr<Node>,
    any_entries: Vector<Ptr<FlowEntry>>,
    key_index: usize,
}

#[derive(Clone, Copy)]
struct Node {
    left: Option<Ptr<Node>>,
    right: Option<Ptr<Node>>,
    entries: Vector<Ptr<FlowEntry>>,
}


impl RadixTree {
    pub fn new(key_index: usize) -> Self {
        let root = Ptr::new(Node {
            left: None,
            right: None,
            entries: Vector::new(0, 8),
        });

        RadixTree {
            root,
            any_entries: Vector::new(8, 8),
            key_index,
        }
    }


    pub fn search(&self, pkt: *const u8, len: usize) -> &Vector<Ptr<FlowEntry>> {
        let mut entries = None;
        let mut node = &self.root;

        for i in 0..len as isize {
            for j in 0..8 {
                let flag = 1 << (7 - j);

                if (unsafe { *pkt.offset(i) } & flag) == flag {
                    if node.entries.len() > 0 {
                        entries = Some(&node.entries);
                    }
                    if node.right.is_none() {
                        if entries.is_none() {
                            return &self.any_entries;
                        }
                        return entries.as_ref().unwrap();
                    }
                    node = &node.right.as_ref().unwrap();
                } else {
                    if node.entries.len() > 0 {
                        entries = Some(&(*node).entries);
                    }
                    if node.left.is_none() {
                        if entries.is_none() {
                            return &self.any_entries;
                        }
                        return entries.as_ref().unwrap();
                    }
                    node = &node.left.as_ref().unwrap();
                }
            }
        }

        &self.any_entries
    }


    pub fn add(&mut self, entry: Ptr<FlowEntry>) {
        let value = entry.values[self.key_index].value;
        if value.is_none() {
            self.any_entries.push(entry);
            return;
        }

        let value = value.unwrap();
        let mut node = self.root;

        for i in 0..value.len() - 1 {
            for j in 0..8 {
                let flag = 1 << (7 - j);
                if (value[i] & flag) == flag {
                    if node.right.is_none() {
                        node.right = Some(Ptr::new(Node {
                            left: None,
                            right: None,
                            entries: Vector::new(0, 8),
                        }));
                    }
                    node = node.right.unwrap();
                } else {
                    if node.left.is_none() {
                        node.left = Some(Ptr::new(Node {
                            left: None,
                            right: None,
                            entries: Vector::new(0, 8),
                        }));
                    }
                    node = node.left.unwrap();
                }
            }
        }


        let byte = value[value.len() - 1];
        let prefix_mask = entry.values[self.key_index].prefix_mask;
        for j in 0..8 {
            let flag = 1 << (7 - j);
            if (prefix_mask & flag) != flag {
                break;
            }

            if (byte & flag) == flag {
                if node.right.is_none() {
                    node.right = Some(Ptr::new(Node {
                        left: None,
                        right: None,
                        entries: Vector::new(0, 8),
                    }));
                }
                node = node.right.unwrap();
            } else {
                if node.left.is_none() {
                    node.left = Some(Ptr::new(Node {
                        left: None,
                        right: None,
                        entries: Vector::new(0, 8),
                    }));
                }
                node = node.left.unwrap();
            }
        }

        node.entries.push(entry);
    }


    pub fn remove(&self) {

    }
}


#[cfg(test)]
mod tests {
    use crate::core::memory::array::Array;
    use crate::core::memory::ptr::Ptr;
    use super::RadixTree;
    use crate::core::helper::linux;
    use crate::pipeline::table::FlowEntry;
    use crate::pipeline::table::ActionSet;
    use crate::pipeline::table::MatchFieldValue;

    #[test]
    fn test_radix_tree() {
        linux::init();

        let mut tree = RadixTree::new(0);

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


        // init pkt
        let mut pkt: Vec<u8> = Vec::new();
        pkt.push(10);
        pkt.push(10);
        pkt.push(10);
        pkt.push(10);
        let pkt_ptr = pkt.as_ptr() as *mut u8;


        // 10.0.0.0/8
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
            value: Some(Array::new(1)),
            prefix_mask: 0xff, 
        };
        entry2.values[0].value.unwrap().init(0, 10);
        tree.add(Ptr::new(entry2));
        assert_eq!(tree.search(pkt_ptr, 4).len(), 1);


        // 172.16.0.0/16
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
            value: Some(Array::new(2)),
            prefix_mask: 0xff, 
        };
        entry3.values[0].value.unwrap().init(0, 172);
        entry3.values[0].value.unwrap().init(1, 16);
        tree.add(Ptr::new(entry3));
        assert_eq!(tree.search(pkt_ptr, 4).len(), 1);
        pkt[0] = 172;
        pkt[1] = 16;
        let entris = tree.search(pkt_ptr, 4); 
        assert_eq!(entris.len(), 1);
        assert_eq!(entris[0].action.action_id, 2);


        //  10.0.0.0/8
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
            value: Some(Array::new(1)),
            prefix_mask: 0xff, 
        };
        entry4.values[0].value.unwrap().init(0, 10);
        tree.add(Ptr::new(entry4));
        pkt[0] = 10;
        let entris = tree.search(pkt_ptr, 4); 
        assert_eq!(entris[0].action.action_id, 1);
        assert_eq!(entris.len(), 2);
        assert_eq!(entris[1].action.action_id, 3);


        // 192.168.0.128/25
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
            value: Some(Array::new(4)),
            prefix_mask: 0x80, 
        };
        entry5.values[0].value.unwrap().init(0, 192);
        entry5.values[0].value.unwrap().init(1, 168);
        entry5.values[0].value.unwrap().init(2, 0);
        entry5.values[0].value.unwrap().init(3, 128);
        tree.add(Ptr::new(entry5));
        pkt[0] = 192;
        pkt[1] = 168;
        pkt[2] = 0;
        pkt[3] = 129;
        let entris = tree.search(pkt_ptr, 4); 
        assert_eq!(entris.len(), 1);
        assert_eq!(entris[0].action.action_id, 4);
    }

}
