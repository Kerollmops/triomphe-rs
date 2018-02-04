extern crate linked_hash_map;

use std::hash::{Hash, BuildHasher};
use std::collections::BTreeMap;
use std::collections::hash_map::RandomState;
use linked_hash_map::LinkedHashMap;

enum Insertion<K, V> {
    Replace(V),
    Evict(K, V),
    Nothing,
}

impl<K, V> Insertion<K, V> {
    fn replace(self) -> Option<V> {
        match self {
            Insertion::Replace(v) => Some(v),
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

    fn insert(&mut self, k: K, v: V) -> Insertion<K, V> {
        if let Some(v) = self.map.insert(k, v) {
            Insertion::Replace(v)
        }
        else if self.map.len() > self.max_size {
            let (k, v) = self.map.pop_front().unwrap();
            Insertion::Evict(k, v)
        }
        else {
            Insertion::Nothing
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

        // TODO test if the key is in ghost_lru/lfu
        //      remove it and update target_sizes

        if self.lru.map.contains_key(&k) {
            match self.lfu.insert(k, v) {
                Insertion::Replace(v) => Some(v),
                Insertion::Evict(k, _) => {
                    self.ghost_lfu.insert(k, ());
                    None
                },
                Insertion::Nothing => None,
            }
        } else {
            match self.lru.insert(k, v) {
                Insertion::Replace(v) => Some(v),
                Insertion::Evict(k, _) => {
                    self.ghost_lru.insert(k, ());
                    None
                },
                Insertion::Nothing => None,
            }
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
