//! Binary-effect observations are distinct from ordinary image/SVG/filter cache hits.

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use skia_safe::{ColorFilter, Image};

use super::cache::{BoundedCache, CacheKey, NativeSkiaCacheUsage};

#[derive(Clone)]
pub(super) enum CachedImageResource {
    Decoded {
        image: Image,
        binary_requested: bool,
        binary_preprocessed: bool,
    },
    SvgPng(Arc<Vec<u8>>),
    Effect(Option<ColorFilter>),
}

impl CachedImageResource {
    fn is_binary_effect(&self) -> bool {
        matches!(
            self,
            Self::Decoded {
                binary_requested: true,
                ..
            }
        )
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct ImageReplayCounters {
    pub binary_cache: NativeSkiaCacheUsage,
    pub preprocessing_failures: usize,
    pub fallback_to_filter: usize,
    pub preprocessed_bytes: usize,
    pub tile_cap_hits: usize,
}

pub(super) struct ImageResourceCache {
    cache: RefCell<BoundedCache<CachedImageResource>>,
    observations: Cell<ImageReplayCounters>,
}

impl ImageResourceCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            cache: RefCell::new(BoundedCache::new(max_entries, max_bytes)),
            observations: Cell::new(ImageReplayCounters::default()),
        }
    }

    pub fn clear(&self) {
        self.cache.borrow_mut().clear();
        self.observations.set(ImageReplayCounters::default());
    }

    pub fn usage(&self) -> NativeSkiaCacheUsage {
        self.cache.borrow().usage()
    }

    pub fn observations(&self) -> ImageReplayCounters {
        self.observations.get()
    }

    pub fn get(&self, key: Option<CacheKey>, binary: bool) -> Option<CachedImageResource> {
        let cached = key.and_then(|key| self.cache.borrow_mut().get(key));
        if binary {
            let mut counters = self.observations.get();
            if cached.is_some() {
                counters.binary_cache.hits = counters.binary_cache.hits.saturating_add(1);
            } else {
                counters.binary_cache.misses = counters.binary_cache.misses.saturating_add(1);
                if key.is_none() {
                    counters.binary_cache.skipped_oversized =
                        counters.binary_cache.skipped_oversized.saturating_add(1);
                }
            }
            self.observations.set(counters);
        }
        cached
    }

    pub fn insert(&self, key: CacheKey, value: CachedImageResource, bytes: usize) {
        let binary = value.is_binary_effect();
        let mut counters = self.observations.get();
        let mut cache = self.cache.borrow_mut();
        let inserted = cache.insert_observed(key, value, bytes, |evicted, _| {
            if evicted.is_binary_effect() {
                counters.binary_cache.evictions = counters.binary_cache.evictions.saturating_add(1);
            }
        });
        if binary && !inserted {
            counters.binary_cache.skipped_oversized =
                counters.binary_cache.skipped_oversized.saturating_add(1);
        }
        let (entries, bytes) = cache.matching_usage(CachedImageResource::is_binary_effect);
        counters.binary_cache.entries = entries;
        counters.binary_cache.approximate_bytes = bytes;
        self.observations.set(counters);
    }

    pub fn record_preprocessing(&self, bytes: Option<usize>) {
        let mut counters = self.observations.get();
        if let Some(bytes) = bytes {
            counters.preprocessed_bytes = counters.preprocessed_bytes.saturating_add(bytes);
        } else {
            counters.preprocessing_failures = counters.preprocessing_failures.saturating_add(1);
        }
        self.observations.set(counters);
    }

    pub fn record_filter_fallback(&self) {
        let mut counters = self.observations.get();
        counters.fallback_to_filter = counters.fallback_to_filter.saturating_add(1);
        self.observations.set(counters);
    }

    pub fn record_tile_cap(&self) {
        let mut counters = self.observations.get();
        counters.tile_cap_hits = counters.tile_cap_hits.saturating_add(1);
        self.observations.set(counters);
    }
}
