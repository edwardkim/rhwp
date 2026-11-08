//! Reservation for the sole request-bound FontInfo copy-on-write path.
//! Arc-owned programs, lazy CMaps, TrueType maps and Unicode memo stay shared.
use super::{allocation::FontAllocation, CIDToGIDMap, Encoding, FontInfo};
use std::collections::HashMap;

impl FontInfo {
    /// Reserve owned clone payload before Arc::make_mut can copy it. HashMap's
    /// clone retains spare buckets, so count capacity rather than only len.
    /// This is requested payload/work, not allocator headers or a live-heap cap.
    pub(crate) fn reserve_cow_copy(&self) -> Option<()> {
        let a = FontAllocation(self.import_budget.as_deref());
        a.check()?;
        a.bytes(std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>())?;
        a.bytes(self.base_font.len())?;
        a.bytes(self.subtype.len())?;
        match &self.encoding {
            Encoding::Standard(name) => a.bytes(name.len())?,
            Encoding::Custom(map) => a.entries(map.capacity())?,
            Encoding::Identity => (),
        }
        if let Some(widths) = &self.widths {
            a.entries(widths.len())?;
        }
        if let Some(CIDToGIDMap::Explicit(map)) = &self.cid_to_gid_map {
            a.entries(map.len())?;
        }
        if let Some(info) = &self.cid_system_info {
            a.bytes(info.registry.len())?;
            a.bytes(info.ordering.len())?;
        }
        if let Some(name) = &self.cid_font_type {
            a.bytes(name.len())?;
        }
        fn map<K, V>(a: FontAllocation<'_>, map: &Option<HashMap<K, V>>) -> Option<()> {
            if let Some(map) = map {
                a.entries(map.capacity())?;
            }
            Some(())
        }
        map(a, &self.cid_widths)?;
        map(a, &self.cff_gid_map)?;
        map(a, &self.cff_cid_to_gid)?;
        map(a, &self.cid_vertical_metrics)?;
        for map in [&self.multi_char_map, &self.diff_glyph_names] {
            a.entries(map.capacity())?;
            for value in map.values() {
                a.bytes(value.len())?;
            }
        }
        if let Some(Some(names)) = self.embedded_glyph_names.get() {
            a.entries(names.len())?;
            for name in names.iter().flatten() {
                a.bytes(name.len())?;
            }
        }
        Some(())
    }
}
