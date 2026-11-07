//! Actual GPU tile surfaces and their owner-thread target reservations.

use std::cell::RefCell;
use std::rc::Rc;

use serde::Serialize;
use skia_safe::Surface;

use super::{limit, NativeGlPlacement, NativeGlPresenterConfig, NativeGlTarget};
use crate::native_host::presentation::NativePresentationStamp;
use crate::native_host::render_snapshot::{profile_name, NativeRenderSnapshot};
use crate::native_host::NativeHostError;

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlTargetStats {
    pub targets: usize,
    pub estimated_bytes: usize,
    pub peak_targets: usize,
    pub peak_estimated_bytes: usize,
    pub rejected_admissions: u64,
}

struct TargetState {
    max_targets: usize,
    max_bytes: usize,
    usage: NativeGlTargetStats,
}

#[derive(Clone)]
pub(super) struct TargetBudget(Rc<RefCell<TargetState>>);

impl TargetBudget {
    pub fn new(max_targets: usize, max_bytes: usize) -> Self {
        Self(Rc::new(RefCell::new(TargetState {
            max_targets,
            max_bytes,
            usage: NativeGlTargetStats::default(),
        })))
    }

    pub fn stats(&self) -> NativeGlTargetStats {
        self.0.borrow().usage
    }

    pub fn can_reserve(&self, count: usize, bytes: usize) -> bool {
        let state = self.0.borrow();
        state
            .usage
            .targets
            .checked_add(count)
            .is_some_and(|total| total <= state.max_targets)
            && state
                .usage
                .estimated_bytes
                .checked_add(bytes)
                .is_some_and(|total| total <= state.max_bytes)
    }

    pub fn reserve(&self, bytes: usize) -> Result<TargetAllocation, NativeHostError> {
        if !self.can_reserve(1, bytes) {
            let mut state = self.0.borrow_mut();
            state.usage.rejected_admissions = state.usage.rejected_admissions.saturating_add(1);
            return Err(limit("GL managed target quota exceeded"));
        }
        let mut state = self.0.borrow_mut();
        state.usage.targets += 1;
        state.usage.estimated_bytes += bytes;
        state.usage.peak_targets = state.usage.peak_targets.max(state.usage.targets);
        state.usage.peak_estimated_bytes = state
            .usage
            .peak_estimated_bytes
            .max(state.usage.estimated_bytes);
        Ok(TargetAllocation {
            budget: self.clone(),
            bytes,
        })
    }
}

pub(super) struct TargetAllocation {
    budget: TargetBudget,
    bytes: usize,
}

impl Drop for TargetAllocation {
    fn drop(&mut self) {
        // A failed borrow leaves the credits retained and the owner fails
        // closed; never free a credit while another operation mutates it.
        if let Ok(mut state) = self.budget.0.try_borrow_mut() {
            state.usage.targets -= 1;
            state.usage.estimated_bytes -= self.bytes;
        }
    }
}

pub(super) struct ManagedTarget {
    // Surface drops before its reservation, including through an Rc tile pin.
    pub surface: Surface,
    pub width: i32,
    pub height: i32,
    allocation: TargetAllocation,
}

impl ManagedTarget {
    pub fn new(surface: Surface, width: i32, height: i32, allocation: TargetAllocation) -> Self {
        Self {
            surface,
            width,
            height,
            allocation,
        }
    }

    pub fn bytes(&self) -> usize {
        self.allocation.bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TileRegion {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl TileRegion {
    pub fn bytes(self) -> usize {
        self.width as usize * self.height as usize * 4
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct TileKey {
    pub region: TileRegion,
    context_id: u64,
    device_generation: u64,
    document_id: u64,
    revision: u64,
    font_generation: u64,
    font_fingerprint: String,
    page: u32,
    profile: &'static str,
    page_width_bits: u64,
    page_height_bits: u64,
    output_options: crate::paint::LayerOutputOptions,
    raster_text_geometry: bool,
    raster_clip_coverage: bool,
    scale_bits: u64,
    phase_x_bits: u32,
    phase_y_bits: u32,
    background_color: u32,
}

pub(super) struct TilePlan {
    pub width: i32,
    pub height: i32,
    pub canonical_bytes: usize,
    pub integer_x: i32,
    pub integer_y: i32,
    pub phase_x: f64,
    pub phase_y: f64,
    pub keys: Vec<TileKey>,
}

impl TilePlan {
    pub fn new(
        config: NativeGlPresenterConfig,
        stamp: NativePresentationStamp,
        target: NativeGlTarget,
        placement: NativeGlPlacement,
        snapshot: &NativeRenderSnapshot,
        clear_color: u32,
    ) -> Option<Self> {
        // COLORREF Some is opaque RGB in the existing strict API. None and a
        // partial-alpha viewport clear retain the original P90 replay path.
        let background_color = placement.background_color?;
        if config.max_cached_tiles == 0 || config.max_tile_bytes == 0 || clear_color >> 24 != 255 {
            return None;
        }
        // Use the exact f32 placement passed to Skia, without phase rounding.
        let origin_x = f64::from(placement.origin_x as f32);
        let origin_y = f64::from(placement.origin_y as f32);
        let integer_x = origin_x.floor() as i32;
        let integer_y = origin_y.floor() as i32;
        let phase_x = origin_x - f64::from(integer_x);
        let phase_y = origin_y - f64::from(integer_y);
        let scale = placement.scale as f32;
        let options = snapshot.tree().output_options;
        let width = f64::from(snapshot.tree().page_width as f32 * scale + phase_x as f32).ceil();
        let height = f64::from(snapshot.tree().page_height as f32 * scale + phase_y as f32).ceil();
        if width <= 0.0
            || height <= 0.0
            || width > f64::from(config.max_dimension)
            || height > f64::from(config.max_dimension)
        {
            return None;
        }
        let (width, height) = (width as i32, height as i32);
        let canonical_bytes = config
            .target_bytes(NativeGlTarget {
                width,
                height,
                ..target
            })
            .ok()?;
        let left = (-i64::from(integer_x)).clamp(0, i64::from(width)) as i32;
        let top = (-i64::from(integer_y)).clamp(0, i64::from(height)) as i32;
        let right =
            (i64::from(target.width) - i64::from(integer_x)).clamp(0, i64::from(width)) as i32;
        let bottom =
            (i64::from(target.height) - i64::from(integer_y)).clamp(0, i64::from(height)) as i32;
        if left >= right || top >= bottom {
            return None;
        }
        let side = config.tile_size as i32;
        let mut keys = Vec::new();
        let mut tile_bytes = 0usize;
        for y in ((top / side * side)..bottom).step_by(config.tile_size) {
            for x in ((left / side * side)..right).step_by(config.tile_size) {
                let region = TileRegion {
                    x,
                    y,
                    width: side.min(width - x),
                    height: side.min(height - y),
                };
                tile_bytes = tile_bytes.checked_add(region.bytes())?;
                if keys.len() >= config.max_cached_tiles || tile_bytes > config.max_tile_bytes {
                    return None;
                }
                keys.push(TileKey {
                    region,
                    context_id: target.context_id,
                    device_generation: stamp.device_generation,
                    document_id: stamp.document_id,
                    revision: stamp.revision,
                    font_generation: stamp.font_generation,
                    font_fingerprint: snapshot.font_state().fingerprint.clone(),
                    page: snapshot.page(),
                    profile: profile_name(snapshot.profile()),
                    page_width_bits: snapshot.tree().page_width.to_bits(),
                    page_height_bits: snapshot.tree().page_height.to_bits(),
                    output_options: options,
                    raster_text_geometry: snapshot.tree().preserves_raster_text_geometry(),
                    raster_clip_coverage: snapshot.tree().uses_raster_clip_coverage(),
                    scale_bits: placement.scale.to_bits(),
                    phase_x_bits: (phase_x as f32).to_bits(),
                    phase_y_bits: (phase_y as f32).to_bits(),
                    background_color,
                });
            }
        }
        Some(Self {
            width,
            height,
            canonical_bytes,
            integer_x,
            integer_y,
            phase_x,
            phase_y,
            keys,
        })
    }
}

pub(super) struct CachedTile {
    pub key: TileKey,
    pub target: RefCell<ManagedTarget>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlTileStats {
    pub cached_tiles: usize,
    pub cached_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub evicted_bytes: u64,
    pub canonical_replays: u64,
    pub canonical_failures: u64,
    pub viewport_fallbacks: u64,
    pub pinned_admission_fallbacks: u64,
    pub target_admission_fallbacks: u64,
}

#[derive(Default)]
pub(super) struct TileCache {
    entries: Vec<Rc<CachedTile>>,
    pub observations: NativeGlTileStats,
}

impl TileCache {
    pub fn stats(&self) -> NativeGlTileStats {
        NativeGlTileStats {
            cached_tiles: self.entries.len(),
            cached_bytes: self
                .entries
                .iter()
                .map(|tile| tile.key.region.bytes())
                .sum(),
            ..self.observations
        }
    }

    pub fn get(&mut self, key: &TileKey) -> Option<Rc<CachedTile>> {
        let Some(index) = self.entries.iter().position(|entry| &entry.key == key) else {
            self.observations.misses = self.observations.misses.saturating_add(1);
            return None;
        };
        self.observations.hits = self.observations.hits.saturating_add(1);
        let tile = self.entries.remove(index);
        let result = Rc::clone(&tile);
        self.entries.push(tile);
        Some(result)
    }

    pub fn insert(&mut self, tile: Rc<CachedTile>) {
        self.entries.push(tile);
    }

    pub fn has_room(&self, config: NativeGlPresenterConfig, count: usize, bytes: usize) -> bool {
        let stats = self.stats();
        stats
            .cached_tiles
            .checked_add(count)
            .is_some_and(|total| total <= config.max_cached_tiles)
            && stats
                .cached_bytes
                .checked_add(bytes)
                .is_some_and(|total| total <= config.max_tile_bytes)
    }

    /// Only unpinned owner-thread entries may be destroyed while GL is current.
    pub fn evict_idle(&mut self) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| Rc::strong_count(entry) == 1)
        else {
            return false;
        };
        let entry = self.entries.remove(index);
        self.observations.evictions = self.observations.evictions.saturating_add(1);
        self.observations.evicted_bytes = self
            .observations
            .evicted_bytes
            .saturating_add(entry.key.region.bytes() as u64);
        drop(entry);
        true
    }

    pub fn clear(&mut self) {
        self.observations.evictions = self
            .observations
            .evictions
            .saturating_add(self.entries.len() as u64);
        self.observations.evicted_bytes = self
            .observations
            .evicted_bytes
            .saturating_add(self.stats().cached_bytes as u64);
        self.entries.clear();
    }
}
