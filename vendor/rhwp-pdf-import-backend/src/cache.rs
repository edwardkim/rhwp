//! General-purpose caching utilities.
//!
//! This module contains cache types and helper traits that are used across
//! multiple modules (e.g., `document`, `fonts::cmap`).

use std::collections::HashMap;
use std::sync::Mutex;

/// Extension trait for Mutex to recover from poisoned locks.
///
/// When a thread panics while holding a Mutex, the lock becomes "poisoned"
/// and the standard `.lock().unwrap()` would cascade panics. This trait
/// provides `.lock_or_recover()` which discards the poison flag and returns
/// the inner data, since the Mutexes in PdfDocument protect caches and
/// bookkeeping (not safety-critical invariants).
pub(crate) trait MutexExt<T> {
    /// Lock the mutex, recovering from poison if needed.
    fn lock_or_recover(&self) -> std::sync::MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| {
            log::debug!("Mutex was poisoned, recovering");
            poisoned.into_inner()
        })
    }
}

/// Entry-count-bounded cache with LRU-style eviction.
///
/// Used for caches where estimating per-entry byte size is impractical
/// (e.g., `Vec<TextSpan>`, `Vec<PdfImage>`). On insert, when at capacity,
/// the least-recently-used entry is evicted. `get()` promotes the accessed
/// key to the most-recently-used position so hot entries survive eviction.
pub(crate) struct BoundedEntryCache<K: Eq + std::hash::Hash + Copy, V> {
    map: HashMap<K, V>,
    insertion_order: std::collections::VecDeque<K>,
    max_entries: usize,
    budget: Option<std::sync::Arc<crate::import_limits::ImportBudget>>,
}

impl<K: Eq + std::hash::Hash + Copy, V> BoundedEntryCache<K, V> {
    pub(crate) fn new(max_entries: usize) -> Self {
        Self::with_budget(max_entries, None)
    }

    pub(crate) fn with_budget(
        max_entries: usize,
        budget: Option<std::sync::Arc<crate::import_limits::ImportBudget>>,
    ) -> Self {
        Self {
            map: HashMap::new(),
            insertion_order: std::collections::VecDeque::new(),
            max_entries,
            budget,
        }
    }

    pub(crate) fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            if let Some(pos) = self.insertion_order.iter().position(|k| k == key) {
                self.insertion_order.remove(pos);
            }
            self.insertion_order.push_back(*key);
        }
        self.map.get(key)
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        // The originating request owns cache index work too. Keep the void
        // legacy API; bounded callers observe the sticky failure before returning.
        if let Some(budget) = &self.budget {
            if budget.charge_entries(1).is_err() {
                return;
            }
            let grow_map = self.map.len() == self.map.capacity() && !self.map.contains_key(&key);
            let grow_order = self.insertion_order.len() == self.insertion_order.capacity()
                && !self.map.contains_key(&key);
            // Reserve the complete replacement storage, not just its delta.
            let bytes = (if grow_map {
                self.map
                    .capacity()
                    .saturating_add(1)
                    .saturating_mul(2)
                    .saturating_mul(std::mem::size_of::<(K, V)>())
            } else {
                0
            })
            .saturating_add(if grow_order {
                self.insertion_order
                    .capacity()
                    .saturating_add(1)
                    .saturating_mul(2)
                    .saturating_mul(std::mem::size_of::<K>())
            } else {
                0
            });
            if budget.charge_decoded(bytes).is_err() {
                return;
            }
            if (grow_map && self.map.try_reserve(1).is_err())
                || (grow_order && self.insertion_order.try_reserve(1).is_err())
            {
                budget.reject("cache index allocation");
                return;
            }
        }
        use std::collections::hash_map::Entry;
        // On re-insert: replace value and promote to most-recently-used position
        // so LRU eviction order stays accurate.
        if let Entry::Occupied(mut e) = self.map.entry(key) {
            e.insert(value);
            if let Some(pos) = self.insertion_order.iter().position(|k| k == &key) {
                self.insertion_order.remove(pos);
            }
            self.insertion_order.push_back(key);
            return;
        }
        // Evict oldest entries if at capacity
        while self.map.len() >= self.max_entries && !self.insertion_order.is_empty() {
            if let Some(old_key) = self.insertion_order.pop_front() {
                self.map.remove(&old_key);
            }
        }
        self.map.insert(key, value);
        self.insertion_order.push_back(key);
    }

    pub(crate) fn clear(&mut self) {
        self.map.clear();
        self.insertion_order.clear();
    }

    pub(crate) fn len(&self) -> usize {
        self.map.len()
    }
}
