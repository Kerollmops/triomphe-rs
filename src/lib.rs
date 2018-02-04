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

impl<K, V> Insert<K, V> {
    fn replace(self) -> Option<V> {
        match self {
            Insert::Replace(v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct PseudoLru<K: Eq + Hash, V, S: BuildHasher = RandomState> {
    map: LinkedHashMap<K, V, S>,
    max_size: usize,
    target_size: usize,
}

impl<K: Eq + Hash, V> PseudoLru<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a capacity of zero is invalid");

        PseudoLru {
            map: LinkedHashMap::new(),
            max_size: capacity,
            target_size: capacity,
        }
    }

    fn insert(&mut self, k: K, v: V) -> Insert<K, V> {
        if let Some(v) = self.map.insert(k, v) {
            Insert::Replace(v)
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

// FIXME capacity badly set !
impl<K: Eq + Hash, V> Arc<K, V> {
    pub fn new(capacity: usize) -> Self {
        Arc {
            ghost_lru: PseudoLru::new(capacity),
            lru: PseudoLru::new(capacity),
            lfu: PseudoLru::new(capacity),
            ghost_lfu: PseudoLru::new(capacity),
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.lru.map.contains_key(&k) {
            self.lfu.insert(k, v).replace()
        }
        else if self.lfu.map.contains_key(&k) {
            self.lfu.insert(k, v).replace()
        }
        else {
            self.lru.insert(k, v).replace()
        }
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
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
