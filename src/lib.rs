extern crate linked_hash_map;

use std::hash::{Hash, BuildHasher};
use std::collections::BTreeMap;
use std::collections::hash_map::RandomState;
use linked_hash_map::LinkedHashMap;

enum Insert<K, V> {
    Replace(V),
    Evict(K, V),
    Nothing,
}

#[derive(Clone)]
struct PseudoLru<K: Eq + Hash, V, S: BuildHasher = RandomState> {
    map: LinkedHashMap<K, V, S>,
    max_size: usize,
}

impl<K: Eq + Hash, V> PseudoLru<K, V> {
    fn new(capacity: usize) -> Self {
        PseudoLru {
            map: LinkedHashMap::new(),
            max_size: capacity,
        }
    }

    fn insert(&mut self, k: K, v: V) -> Insert<K, V> {
        if let Some(old) = self.map.insert(k, v) {
            Insert::Replace(old)
        }
        else if self.map.len() > self.max_size {
            let (k, v) = self.map.pop_front().unwrap();
            Insert::Evict(k, v)
        }
        else {
            Insert::Nothing
        }
    }
}

#[derive(Clone)]
pub struct Arc<K: Eq + Hash, V, S: BuildHasher = RandomState> {
    ghost_lru: PseudoLru<K, (), S>,
    lru: PseudoLru<K, V, S>,
    lfu: PseudoLru<K, V, S>,
    ghost_lfu: PseudoLru<K, (), S>,
}

impl<K: Eq + Hash, V> Arc<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a capacity of zero is invalid");

        Arc {
            ghost_lru: PseudoLru::new(capacity),
            lru: PseudoLru::new(capacity),
            lfu: PseudoLru::new(capacity),
            ghost_lfu: PseudoLru::new(capacity),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        if self.lru.map.contains_key(&k) {
            if let None = self.lfu.map.get_refresh(&k) {
                if let Insert::Evict(k, _) = self.lfu.insert(k, v) {
                    self.ghost_lfu.insert(k, ());
                    // TODO change capacities ???
                }
            }
        }
        else if let Insert::Evict(k, _) = self.lru.insert(k, v) {
            self.ghost_lru.insert(k, ());
            // TODO change capacities ???
        }
    }

    pub fn get_mut(&mut self, k: &K) -> Option<V> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
