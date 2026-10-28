//! One bounded CPU page snapshot per native session. This cache is optional;
//! misses and rejected retention never change the region pixel contract.

use serde::Serialize;

use super::{NativeDocumentSession, NativeFrame, NativeHostError, MAX_FRAME_PIXELS};
use crate::renderer::layer_renderer::RasterRenderOptions;
use crate::renderer::raster_pixels::{RasterRegion, RasterRegionPixels};
use crate::renderer::skia::{PreparedSkiaPage, SkiaRegionPlan};

pub const DEFAULT_REGION_CACHE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_REGION_CACHE_BYTES: usize = 64 * 1024 * 1024;

/// Counters accumulate for the session lifetime, including across cache clears.
/// `replays` counts successful full-page rasterizations for region requests.
/// Hits count successful requests served by the matching retained snapshot;
/// `off_page_requests` counts successful clear-only requests on a cold key.
/// Gauges count only the retained N32 image, excluding output buffers, renderer
/// resources, layout data and Skia intermediates. This is not an RSS estimate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRegionCacheStats {
    pub entries: usize,
    pub retained_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub replays: u64,
    pub evictions: u64,
    pub invalidations: u64,
    pub bypassed_oversized: u64,
    pub off_page_requests: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RegionKey {
    revision: u64,
    font_generation: u64,
    page: u32,
    scale: u64,
}

struct RegionEntry {
    key: RegionKey,
    page: PreparedSkiaPage,
}

pub(super) struct NativeRegionCache {
    entry: Option<RegionEntry>,
    max_bytes: usize,
    stats: NativeRegionCacheStats,
}

impl Default for NativeRegionCache {
    fn default() -> Self {
        Self {
            entry: None,
            max_bytes: DEFAULT_REGION_CACHE_BYTES,
            stats: NativeRegionCacheStats::default(),
        }
    }
}

impl NativeRegionCache {
    fn evict(&mut self) {
        if self.entry.take().is_some() {
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
    }

    pub(super) fn invalidate(&mut self) {
        self.evict();
        self.stats.invalidations = self.stats.invalidations.saturating_add(1);
    }

    fn stats(&self) -> NativeRegionCacheStats {
        let mut stats = self.stats;
        stats.entries = usize::from(self.entry.is_some());
        stats.retained_bytes = self
            .entry
            .as_ref()
            .map_or(0, |entry| entry.page.retained_bytes());
        stats
    }
}

impl NativeDocumentSession {
    /// Configure retention for one completed CPU page, from zero (disabled) to
    /// 64 MiB. Changing the limit evicts the current entry before returning.
    /// Rejected values leave the existing limit, snapshot and counters intact.
    /// The per-request 128 MiB bitmap peak remains independent of this limit.
    pub fn set_region_cache_limit(&self, max_bytes: usize) -> Result<(), NativeHostError> {
        if max_bytes > MAX_REGION_CACHE_BYTES {
            return Err(NativeHostError::LimitExceeded(
                "native region cache limit exceeds 64 MiB".into(),
            ));
        }
        let mut cache = self.region_cache.borrow_mut();
        if cache.max_bytes != max_bytes {
            cache.invalidate();
            cache.max_bytes = max_bytes;
        }
        Ok(())
    }

    pub fn region_cache_stats(&self) -> NativeRegionCacheStats {
        self.region_cache.borrow().stats()
    }

    pub(super) fn render_region_cached(
        &self,
        page: u32,
        region: RasterRegion,
        scale: f64,
    ) -> Result<NativeFrame, NativeHostError> {
        let options = RasterRenderOptions {
            scale,
            max_pixels: MAX_FRAME_PIXELS,
            transparent: false,
            ..Default::default()
        };
        let key = RegionKey {
            revision: self.revision,
            font_generation: self.font_generation,
            page,
            scale: scale.to_bits(),
        };
        let mut cache = self.region_cache.borrow_mut();
        if let Some(entry) = cache.entry.as_ref().filter(|entry| entry.key == key) {
            let plan = entry.page.plan_region(region)?;
            if entry.page.can_render_plan(plan) {
                let output = entry.page.render_region(region)?;
                cache.stats.hits = cache.stats.hits.saturating_add(1);
                return Ok(self.region_frame(output));
            }
            // An off-page request needs no working raster. Release the old
            // image before its large clear output so retention cannot turn a
            // formerly admissible direct request into a bitmap peak error.
            cache.evict();
        }

        let tree = self
            .core
            .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
        let plan = SkiaRegionPlan::new(tree.page_width, tree.page_height, region, options)?;
        cache.stats.misses = cache.stats.misses.saturating_add(1);
        if plan.intersection().is_none() {
            // The matching-key case above evicts when required. A different-key
            // snapshot must also fit alongside the clear-only output.
            let retained = cache
                .entry
                .as_ref()
                .map_or(0, |entry| entry.page.retained_bytes());
            if !PreparedSkiaPage::fits_bitmap_budget(retained, plan.output_bytes()) {
                cache.evict();
            }
            let (output, prepared) = self
                .renderer()
                .prepare_page_for_region(&tree, region, options)?;
            debug_assert!(prepared.is_none());
            cache.stats.off_page_requests = cache.stats.off_page_requests.saturating_add(1);
            return Ok(self.region_frame(output));
        }

        // Drop the old N32 image before any new surface/output allocation. A
        // replacement never temporarily retains two full-page snapshots.
        cache.evict();
        let (output, prepared) = self
            .renderer()
            .prepare_page_for_region(&tree, region, options)?;
        cache.stats.replays = cache.stats.replays.saturating_add(1);
        if let Some(prepared) = prepared {
            if prepared.retained_bytes() <= cache.max_bytes {
                cache.entry = Some(RegionEntry {
                    key,
                    page: prepared,
                });
            } else {
                cache.stats.bypassed_oversized = cache.stats.bypassed_oversized.saturating_add(1);
            }
        }
        Ok(self.region_frame(output))
    }

    fn region_frame(&self, output: RasterRegionPixels) -> NativeFrame {
        NativeFrame {
            pixels: output.pixels,
            revision: self.revision,
            font_generation: self.font_generation,
            origin_x: output.origin_x,
            origin_y: output.origin_y,
        }
    }
}
