extern crate linked_hash_map;

use std::hash::{Hash, BuildHasher};
use std::collections::BTreeMap;
use std::collections::hash_map::RandomState;
use linked_hash_map::LinkedHashMap;

// TODO rename Insert::{ Replacement, Eviction }
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
    capacity: usize,
}

impl<K: Eq + Hash, V> PseudoLru<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a capacity of zero is invalid");

        PseudoLru {
            map: LinkedHashMap::new(),
            capacity: capacity,
        }
    }

    fn insert(&mut self, k: K, v: V) -> Insertion<K, V> {
        if let Some(v) = self.map.insert(k, v) {
            Insertion::Replace(v)
        }
        else if self.map.len() > self.capacity {
            let (k, v) = self.map.pop_front().unwrap();
            Insertion::Evict(k, v)
        }
        else {
            Insertion::Nothing
        }
    }

    fn increase_capacity(&mut self) {
        unimplemented!()
    }

    fn decrease_capacity(&mut self) -> Option<(K, V)> {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct Arc<K: Eq + Hash, V, S: BuildHasher = RandomState> {
    ghost_lru: PseudoLru<K, (), S>, // B1
    lru: PseudoLru<K, V, S>,        // T1
    lfu: PseudoLru<K, V, S>,        // T2
    ghost_lfu: PseudoLru<K, (), S>, // B2
    partition: usize,               // repartition of L1 and L2 capacities
}

// FIXME capacity badly set !
impl<K: Eq + Hash, V> Arc<K, V> {
    pub fn new(capacity: usize) -> Self {
        Arc {
            ghost_lru: PseudoLru::new(capacity),
            lru: PseudoLru::new(capacity),
            lfu: PseudoLru::new(capacity),
            ghost_lfu: PseudoLru::new(capacity),
            partition: capacity / 2,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {

        // TODO why not removing it from the rlu
        //      and moving it to the ghost_lru ?
        //      it will be in the lfu anyway
        if self.lru.map.contains_key(&k) {

            if self.ghost_lru.map.remove(&k).is_some() {
                // increase the LRU (T1) capacity
                let total = self.lru.capacity + self.lfu.capacity;
                self.partition = (self.partition + 1).min(total);
            }

            // self.lfu.set_capacity(xxx) and catch evicted

            match self.lfu.insert(k, v) {
                Insertion::Replace(v) => Some(v),
                Insertion::Evict(k, _) => {
                    self.ghost_lfu.insert(k, ());
                    None
                },
                Insertion::Nothing => None,
            }
        }
        else {

            if self.ghost_lfu.map.remove(&k).is_some() {
                // increase the LFU (T2) capacity
                self.partition.saturating_sub(1);
            }

            // self.lru.set_capacity(xxx) and catch evicted

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
