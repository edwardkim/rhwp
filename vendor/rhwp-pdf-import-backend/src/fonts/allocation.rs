//! Reservations for font expansion and Unicode copies in the originating request.
//! These are cumulative work counters, not allocator overhead or live-heap limits.
use crate::import_limits::ImportBudget;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Copy)]
pub(crate) struct FontAllocation<'a>(pub(crate) Option<&'a ImportBudget>);

impl FontAllocation<'_> {
    pub(crate) fn check(self) -> Option<()> {
        if let Some(budget) = self.0 {
            budget.check().ok()?;
        }
        Some(())
    }

    pub(crate) fn reject(self) {
        if let Some(budget) = self.0 {
            budget.reject("font allocation");
        }
    }

    pub(crate) fn entries(self, count: usize) -> Option<()> {
        if let Some(budget) = self.0 {
            budget.charge_font_entries(count).ok()?;
        }
        Some(())
    }

    pub(crate) fn bytes(self, count: usize) -> Option<()> {
        if let Some(budget) = self.0 {
            budget.charge_decoded(count).ok()?;
        }
        Some(())
    }

    pub(crate) fn vec<T>(self, count: usize) -> Option<Vec<T>> {
        self.entries(count)?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(count)
            .map_err(|_| self.reject())
            .ok()?;
        Some(value)
    }

    pub(crate) fn map<K: Eq + Hash, V>(self, count: usize) -> Option<HashMap<K, V>> {
        self.entries(count)?;
        let mut value = HashMap::new();
        value.try_reserve(count).map_err(|_| self.reject()).ok()?;
        Some(value)
    }

    pub(crate) fn insert<K: Eq + Hash, V>(
        self,
        map: &mut HashMap<K, V>,
        key: K,
        value: V,
    ) -> Option<()> {
        // Replacements are work too, as in the existing TrueType expansion counter.
        self.entries(1)?;
        if !map.contains_key(&key) {
            reserve_map_growth(self, map)?;
        }
        map.insert(key, value);
        Some(())
    }

    pub(crate) fn string(self, value: &str) -> Option<String> {
        self.string_capacity(value.len()).map(|mut out| {
            out.push_str(value);
            out
        })
    }

    pub(crate) fn string_capacity(self, bytes: usize) -> Option<String> {
        self.check()?;
        if let Some(budget) = self.0 {
            if bytes > budget.limits.string_bytes {
                budget.reject("font string bytes");
                return None;
            }
        }
        self.bytes(bytes)?;
        let mut value = String::new();
        value
            .try_reserve_exact(bytes)
            .map_err(|_| self.reject())
            .ok()?;
        Some(value)
    }

    pub(crate) fn character(self, value: char) -> Option<String> {
        self.string(value.encode_utf8(&mut [0; 4]))
    }
}

/// Resource-name/cache bookkeeping belongs to collection entries, not the
/// independently configured glyph/CID metric expansion counter.
#[derive(Clone, Copy)]
pub(crate) struct CollectionAllocation<'a>(pub(crate) Option<&'a ImportBudget>);

impl CollectionAllocation<'_> {
    pub(crate) fn check(self) -> Option<()> {
        FontAllocation(self.0).check()
    }
    pub(crate) fn bytes(self, bytes: usize) -> Option<()> {
        FontAllocation(self.0).bytes(bytes)
    }
    pub(crate) fn string(self, value: &str) -> Option<String> {
        FontAllocation(self.0).string(value)
    }
    fn slots<T>(self, count: usize) -> Option<()> {
        if let Some(budget) = self.0 {
            budget.charge_entries(count).ok()?;
            let bytes = count.checked_mul(std::mem::size_of::<T>()).or_else(|| {
                budget.reject("font index size overflow");
                None
            })?;
            budget.charge_decoded(bytes).ok()?;
        }
        Some(())
    }
    pub(crate) fn vec<T>(self, count: usize) -> Option<Vec<T>> {
        self.slots::<T>(count)?;
        let mut value = Vec::new();
        value
            .try_reserve_exact(count)
            .map_err(|_| FontAllocation(self.0).reject())
            .ok()?;
        Some(value)
    }
    pub(crate) fn map<K: Eq + Hash, V>(self, count: usize) -> Option<HashMap<K, V>> {
        self.slots::<(K, V)>(count)?;
        let mut value = HashMap::new();
        value
            .try_reserve(count)
            .map_err(|_| FontAllocation(self.0).reject())
            .ok()?;
        Some(value)
    }
    pub(crate) fn insert<K: Eq + Hash, V>(
        self,
        map: &mut HashMap<K, V>,
        key: K,
        value: V,
    ) -> Option<()> {
        self.slots::<(K, V)>(1)?;
        if !map.contains_key(&key) {
            reserve_map_growth(FontAllocation(self.0), map)?;
        }
        map.insert(key, value);
        Some(())
    }
}

fn reserve_map_growth<K: Eq + Hash, V>(
    allocation: FontAllocation<'_>,
    map: &mut HashMap<K, V>,
) -> Option<()> {
    if map.len() == map.capacity() {
        // Count a full new backing table before a growth allocation. The
        // per-entry work reservation above is not a capacity delta credit.
        let bytes = map
            .capacity()
            .checked_add(1)
            .and_then(|n| n.checked_mul(2))
            .map(|n| n.max(4))
            .and_then(|n| n.checked_mul(std::mem::size_of::<(K, V)>()))
            .or_else(|| {
                allocation.reject();
                None
            })?;
        allocation.bytes(bytes)?;
        map.try_reserve(1).map_err(|_| allocation.reject()).ok()?;
    }
    Some(())
}
