extern crate linked_hash_map;

use std::hash::{Hash, BuildHasher};
use std::collections::BTreeMap;
use std::collections::hash_map::RandomState;
use linked_hash_map::LinkedHashMap;

enum Insert<K, V> {
    Replacement(V),
    Eviction(K, V),
    Nothing,
}

impl<K, V> Insert<K, V> {
    fn replace(self) -> Option<V> {
        match self {
            Insert::Replacement(v) => Some(v),
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
        PseudoLru {
            map: LinkedHashMap::new(),
            capacity: capacity,
        }
    }

    fn insert(&mut self, k: K, v: V) -> Insert<K, V> {
        if let Some(v) = self.map.insert(k, v) {
            Insert::Replacement(v)
        }
        else if self.map.len() > self.capacity {
            let (k, v) = self.map.pop_front().unwrap();
            Insert::Eviction(k, v)
        }
        else {
            Insert::Nothing
        }
    }

    fn increase_capacity(&mut self) {
        self.capacity += 1;
    }

    fn decrease_capacity(&mut self) -> Option<(K, V)> {
        self.capacity = self.capacity.saturating_sub(1);
        if self.map.len() > self.capacity {
            self.map.pop_front()
        } else {
            None
        }
    }
}

// TODO remove this, its a little ugly
fn set_capacity<K, V>(lru: &mut PseudoLru<K, V>, target: usize) -> Option<(K, V)>
    where K: Eq + Hash
{
    if lru.capacity > target {
        lru.decrease_capacity()
    } else {
        lru.increase_capacity();
        None
    }
}

#[derive(Clone)]
pub struct Arc<K: Eq + Hash, V, S: BuildHasher = RandomState> {
    ghost_lru: PseudoLru<K, (), S>, // B1
    lru: PseudoLru<K, V, S>,        // T1
    lfu: PseudoLru<K, V, S>,        // T2
    ghost_lfu: PseudoLru<K, (), S>, // B2
    partition: usize,               // repartition of T1 and T2 capacities
}

impl<K: Eq + Hash, V> Arc<K, V> {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity / 2;
        Arc {
            ghost_lru: PseudoLru::new(capacity),
            lru: PseudoLru::new(capacity),
            lfu: PseudoLru::new(capacity),
            ghost_lfu: PseudoLru::new(capacity),
            partition: capacity,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {

        // TODO why not removing it from the rlu
        //      and moving it to the ghost_lru ?
        //      it will be in the lfu anyway
        if self.lru.map.contains_key(&k) {

            let total_capacity = self.lru.capacity + self.lfu.capacity;

            if self.ghost_lru.map.remove(&k).is_some() {
                // increase the LRU (T1) capacity
                self.partition = (self.partition + 1).min(total_capacity);
            }

            // set the LFU capacity and manage the evicted key
            let target_capacity = total_capacity - self.partition;
            if let Some((k, _)) = set_capacity(&mut self.lfu, target_capacity) {
                self.ghost_lfu.insert(k, ());
            }

            match self.lfu.insert(k, v) {
                Insert::Replacement(v) => Some(v),
                Insert::Eviction(k, _) => {
                    self.ghost_lfu.insert(k, ());
                    None
                },
                Insert::Nothing => None,
            }
        }
        else {

            if self.ghost_lfu.map.remove(&k).is_some() {
                // increase the LFU (T2) capacity
                self.partition.saturating_sub(1);
            }

            // set the LRU capacity and manage the evicted key
            if let Some((k, _)) = set_capacity(&mut self.lru, self.partition) {
                self.ghost_lru.insert(k, ());
            }

            match self.lru.insert(k, v) {
                Insert::Replacement(v) => Some(v),
                Insert::Eviction(k, _) => {
                    self.ghost_lru.insert(k, ());
                    None
                },
                Insert::Nothing => None,
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
