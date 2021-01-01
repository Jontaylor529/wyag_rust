use std::{cell::RefCell, hash::Hash, ptr::null};
use std::collections::{HashMap,LinkedList};
use std::ptr::null_mut;
use std::rc::{Rc, Weak};

///Dictionary that remembers the order that keys were added in
#[derive(Debug)]
pub struct OrderedDictionary<K: Hash + Eq + PartialEq + Clone, V> {
    primary_dict: HashMap<K,V>,
    ordered_keys: NodeList<K>,
    secondary_dict: HashMap<K,WeakNode<K>>,
}

impl <K: Hash + Eq + PartialEq + Clone ,V> OrderedDictionary<K,V>{

    pub fn new() -> OrderedDictionary<K,V> {
        OrderedDictionary {
            primary_dict: HashMap::new(),
            ordered_keys: NodeList::new(),
            secondary_dict: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: K, value: V) -> Option<V> {
        if let Some(old_value) = self.primary_dict.insert(key.clone(), value) {
            Some(old_value)
        }else {
            let new_node = self.ordered_keys.insert(key.clone());
            self.secondary_dict.insert(key, new_node);
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let node = self.secondary_dict
        .get(&key)?
        .upgrade()
        .expect("Node dropped unexpectedly");

        self.ordered_keys.remove(node);
        self.secondary_dict.remove(&key);
        self.primary_dict.remove(&key)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.primary_dict.get(key)
    }
}

impl <K: Hash + Eq + PartialEq + Clone ,V> IntoIterator for OrderedDictionary<K, V> {
    type Item = (K,V);
    type IntoIter = OrderedDictIter<K,V>;

    fn into_iter(self) -> Self::IntoIter {
        OrderedDictIter::new(self)
    }
}

pub struct OrderedDictIter<K: Hash + Eq + PartialEq + Clone ,V> {
    current_node: Option<StrongNode<K>>,
    dictionary: OrderedDictionary<K,V>,
}

impl <K: Hash + Eq + PartialEq + Clone ,V> OrderedDictIter<K,V> {
    fn new(dict: OrderedDictionary<K,V>) -> OrderedDictIter<K,V>{
        OrderedDictIter {
            current_node: dict.ordered_keys.root.clone(),
            dictionary: dict,
        }
    }
}

impl <K: Hash + Eq + PartialEq + Clone , V> Iterator for OrderedDictIter<K, V> {
    type Item = (K,V);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.current_node.clone() {
            let key = node.borrow().val.clone();
            let value = self.dictionary.remove(&key).expect("Place in node list desynced from primary dictionary");
            self.current_node = node.borrow().next.clone();
            Some((key,value))
        } else {
            None
        }
    }
}

type StrongNode<T> = Rc<RefCell<KeyNode<T>>>;
type WeakNode<T> = Weak<RefCell<KeyNode<T>>>;

#[derive(Debug)]
struct KeyNode<K> {
    next: Option<StrongNode<K>>,
    prev: Option<WeakNode<K>>,
    val: K,
}
#[derive(Debug)]
struct NodeList<K> {
    root: Option<StrongNode<K>>,
}


impl <K> NodeList<K> {

    fn new() -> NodeList<K> {
        NodeList {
            root: None,
        }
    }

    pub fn insert(&mut self, val: K) -> WeakNode<K> {
        if let Some(node) = self.root.clone() {
            NodeList::append(node, val)
        } else {
            let new_node = Rc::new(
                RefCell::new(
                    KeyNode {
                        next: None,
                        prev: None,
                        val: val,
                    }
                )
            );
            self.root = Some(new_node.clone());
            Rc::downgrade(&new_node)
        }
    }

    pub fn remove(&mut self, node: StrongNode<K>) -> () {
        let next_node = node.borrow_mut().next.clone();
        let prev_node = node.borrow_mut().prev.clone();
        
        if let Some(next_node) = next_node.clone() {
            next_node.borrow_mut().prev = prev_node.clone();
        }

        if let Some(prev_node) = prev_node{
            let prev_node = prev_node.upgrade().expect("Node in linked list was dropped unexpectedly");
            prev_node.borrow_mut().next = next_node;
        }
        //node gets dropped since it is moved in here
    }

    fn append(node: StrongNode<K>, val: K) -> WeakNode<K> {
        let next_node = (*node).borrow_mut().next.clone();
        if let Some(node) = next_node {
            NodeList::append(node, val)
        } else {
            let new_node = Rc::new(
                RefCell::new(
                    KeyNode {
                        next: None,
                        prev: Some(Rc::downgrade(&node)),
                        val: val,
                    }
                )
            );
            (*node).borrow_mut().next = Some(new_node.clone());
            Rc::downgrade(&new_node)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::OrderedDictionary;

    #[test]
    fn creates_ordered_iterator() {
        let pairs = [("test1",9), ("test2", 8), ("test3", 0)];
        let mut test_dict: OrderedDictionary<&str, i32> = OrderedDictionary::new();

        for (k,v) in pairs.iter() {
            test_dict.add(k.clone(), v.clone());
        }
        let mut check_iter = pairs.iter();
        for pair in test_dict {
            println!("Grabbing pair from dictionary...");
            let expected = *check_iter.next().expect("More entries in dictionary than expected");
            assert!(pair == expected,"Mismatch: Dictionary: {:?}, Expected: {:?}",pair,expected);
        }
    }
}